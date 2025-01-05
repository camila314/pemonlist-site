import app from '../app';
import db from '../db';
import { requireLogin } from './account';

app.get('/submit', requireLogin, async (req, res) => {
	if (req.account!.status !== "Done") {
		return res.redirect('/account');
	}

	const levels = await db.query(`
        select Level {
            name, level_id, placement, creator
        } order by .placement limit 150
	`);

    res.render('submit', {levels});
});

app.post('/submit', requireLogin, async (req, res) => {
	const data = await db.querySingle<Record<string, any>>(`
		with lvl := (select Level filter .level_id = <int64>$level_id)
		select {
			level := lvl,
			entry := (
				with entry := (
					select Entry { id, time } filter
					    .video_id = <str>$video_id and
					    .level = lvl and
					    .status != Status.Denied
						limit 1
				) select <Entry>{} if exists entry and (entry.time <= <duration><str>$time) ?? false else (insert Entry {
		            status := Status.Waiting,
		            video_id := <str>$video_id,
		            raw_video := <str>$raw,
		            player := <Player><uuid><str>$player_id,
		            level := lvl,
		            time := <duration><str>$time,
		            mobile := <bool>$mobile,
		            notes := <str>$notes
				})
			)
		}
	`, {
		level_id: parseInt(req.body.levelid) || 0,
		video_id: req.body.videoid || req.body.raw,
		time: req.body.time,
		raw: req.body.raw,
		player_id: req.account!.player!.id,
		mobile: req.body.device === "mobile",
		notes: req.body.notes
	}) ?? {};

	console.log(data);
	if (data.level === null)
		return res.redirect('/submit');
	if (data.entry === null)
		return res.render('duplicate');

	res.render('submitted');
});
