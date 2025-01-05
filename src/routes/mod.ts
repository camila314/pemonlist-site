import app from '../app';
import db, {schema} from '../db';
import {RequestHandler} from 'express';

const requireMod: RequestHandler = async (req, res, next) => {
    if (!req.account)
        return res.redirect('/login');
    if (!req.account!.mod)
        return res.redirect('/account');
    next();
}

app.get('/mod', requireMod, (req, res) => {
    res.render('mod/mod');
});

app.get('/mod/levels', requireMod, async (req, res) => {
    const data = await db.querySingle(`
        select {
            players := (select Player {
                id,
                name
            } order by .name),
            levels := (select Level {
                id,
                name,
                placement,
                video_id,
                level_id,
                creator,
                verifier: { name }
            } order by .placement)
        }
    `);
    res.render('mod/levels', data!);
});
app.post('/mod/addlevel', requireMod, async (req, res) => {
    await db.execute(`
        update Level filter .placement >= <int32><str>$placement set {
            placement := .placement + 1
        };

        insert Level {
            creator := <str>$creator,
            level_id := <int32><str>$level_id,
            name := <str>$level_name,
            placement := <int32>$placement,
            verifier := (
                insert Player {
                    name := <str>$verifier
                } unless conflict on .name else (select Player)
            ),
            video_id := <str>$video_id
        }
    `, {
        verifier: req.body.verifiername,
        creator: req.body.creator,
        level_id: req.body.levelid,
        level_name: req.body.name,
        placement: parseInt(req.body.placement),
        video_id: req.body.videoid
    });

    res.redirect('/mod/levels');
});
app.post('/mod/editlevel', requireMod, async (req, res) => {
    await db.execute(`
        with
            level := (select (<Level><uuid><str>$id) { placement }),
            old_placement := level.placement
        select (
            with
                new_level := (select (update level set {
                    creator := <str>$creator,
                    level_id := <int32><str>$level_id,
                    name := <str>$level_name,
                    verifier := (
                        insert Player {
                            name := <str>$verifier
                        } unless conflict on .name else (select Player)
                    ),
                    video_id := <str>$video_id,
                    placement := <int32>$placement
                }) { placement }),
                new_placement := new_level.placement
            select (
                (update Level filter .id != level.id 
                                 and .placement > level.placement
                                 and .placement <= new_level.placement set {
                    placement := .placement - 1
                }) if old_placement < new_placement else
                ((update Level filter .id != level.id 
                                 and .placement < level.placement
                                 and .placement >= new_level.placement set {
                    placement := .placement + 1
                }) if old_placement > new_placement else {})
            )
        )
    `, {
        id: req.body.id,
        verifier: req.body.verifiername,
        creator: req.body.creator,
        level_id: req.body.levelid,
        level_name: req.body.name,
        video_id: req.body.videoid,
        placement: parseInt(req.body.placement)
    });

    res.redirect('/mod/levels');
});

app.get('/mod/records', requireMod, async (req, res) => {
    let records = await db.query(`
        select Entry {
            id,
            video_id,
            raw_video,
            time_format := (select to_str(.time, "FMHH24:MI:SS")),
            time_ms := (select to_str(.time, "MS")),
            status,
            mobile,
            player: {
                name,
                account := (select .<player[is Account] {
                    image,
                    profile_shape,
                    discord: { global_name, user_id, username, avatar }
                } limit 1)
            },
            level: { name, placement, video_id, level_id },
            notes
        } filter .status != Status.Approved and .status != Status.Denied order by .created_at asc
    `);

    const pageLimit = 10;
    const maxPage = Math.ceil(records.length / pageLimit);
    const page = Math.max(1, Math.min(req.page, maxPage));

    res.render('mod/records', {
        page,
        records: records.slice((page - 1) * pageLimit, page * pageLimit),
        elapsed: Math.round((Date.now() - req.timestamp) / 10) / 100,
        stats: {
            page,
            prev_page: Math.max(1, page - 1),
            next_page: Math.min(maxPage, page + 1),
            last_page: maxPage,
            total: records.length
        }
    });
});
app.post('/mod/records', requireMod, async (req, res) => {
    await db.execute(`
        with entry := (select <Entry><uuid><str>$entry_id { level, player })
        select (
            (update entry set {
                time := <duration><str>$time,
                status := <Status><str>$status,
                mobile := <bool>$mobile,
                mod := <Account><uuid><str>$mod,
                reason := <str>$reason,
            }),

            (delete Entry filter
                .level = entry.level and
                .player = entry.player and
                .status = Status.Approved and
                .status = <Status><str>$status
                .id != $entry_id)
            )
        );
    `, {
        time: req.body.time,
        status: req.body.status.replace(/(.)(?=.+)/, a=>a.toUpperCase()),
        mobile: req.body.device == 'mobile',
        mod: req.account!.id,
        reason: req.body.reason,
        entry_id: req.body.entryid,
    });

    res.redirect('/mod/records');
});


app.get('/mod/users', requireMod, async (req, res) => {
    let migrations = await db.query(`
        select MigrationRequest {
            id,
            account: {
                profile_shape,
                image
            },
            player: {
                name
            },
            discord: {
                global_name,
                user_id,
                avatar,
                username
            }
        } filter .account.status = AccountStatus.Migrating order by .created_at asc
    `);

    res.render('mod/users', {
        migrations,
        elapsed: Math.round((Date.now() - req.timestamp) / 10) / 100,
    });
});
app.post('/mod/users', requireMod, async (req, res) => {
    if (req.body.status === "accept") {
        await db.execute(`
            with migration := (select (<MigrationRequest><uuid><str>$migration_id) { account, discord, player })
            select (
                (update migration.account set {
                    discord := migration.discord,
                    player := migration.player,
                    status := AccountStatus.Done
                }),
                (delete migration)
            )
        `, {
            migration_id: req.body.migrationid
        });
    } else if (req.body.status === "deny") {
        await db.execute(`
            with migration := (select (<MigrationRequest><uuid><str>$migration_id) { account, discord, player })
            select (
                (delete AuthToken filter .account = migration.account),
                (delete migration),
                (delete migration.account),
                (delete migration.discord)
            )
        `, {
            migration_id: req.body.migrationid
        });
    }

    res.redirect('/mod/users');
});
