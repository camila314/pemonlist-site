import app from '../app';
import db from '../db';

const startTime = Date.now();

const uptime = (req, res) => {
	res.json({
		startup: startTime / 1000,
		uptime: (Date.now() - startTime) / 1000,
		tasks: 1,
		workers: 1
	});
};
app.get('/api/uptime', uptime).head('/api/uptime', uptime);

app.get('/api/docs', (req, res) => {
	res.render('docs/index');
});

app.get('/api/docs/endpoints', (req, res) => {
	res.render('docs/endpoints');
});

app.get('/api/level/:id', async (req, res) => {
	const level = await db.querySingle(`
		select Level {
	        placement,
	        level_id,
	        name,
	        creator,
	        verifier: { id, name },
	        video_id,
	        points,
	        records := (select .entries {
	            player: {
	                id,
	                name
	            },
	            timestamp_milliseconds := (select duration_get(<cal::relative_duration>.time, "milliseconds")),
	            formatted_time := (select to_str(.time, "FMHH24:MI:SS.MS")),
	            video_id,
	            mobile,
	            rank
	        } filter .status = Status.Approved order by .time)
	    } filter .level_id = <int64>$id
	`, {
		id: parseInt(req.params.id)
	});

	if (!level) {
		res.status(400).json({
			error: true,
			code: 'bad_level_id'
		});
	} else {
		res.json(level);
	}
});

app.get('/api/list', async (req, res) => {
	if (req.query.version === "1") {
		res.json(await db.query(`
			select Level {
	            placement,
	            level_id,
	            name,
	            creator,
	            verifier: { id, name },
	            video_id,
	            top_record := (select .entries {
	                player: {
	                    id,
	                    name
	                },
	                timestamp_milliseconds := (select duration_get(<cal::relative_duration>.time, "milliseconds")),
	                formatted_time := (select to_str(.time, "FMHH24:MI:SS.MS"))
	            } filter .status = Status.Approved  order by .time limit 1)
	        } order by .placement offset {} limit {}
		`));
		return;
	}

	const limit = parseInt(req.query.limit as string) || 10;

	if (req.page <= 0) {
		res.status(400).json({
			error: true,
			code: 'bad_page'
		});
		return;
	}
	if (limit < 0) {
		res.status(400).json({
			error: true,
			code: 'bad_limit'
		});
		return;
	}

	res.json(await db.query(`
		select {
			data := (
				select Level {
			        placement,
			        level_id,
			        name,
			        creator,
			        verifier: { id, name },
			        video_id,
			        top_record := (select .entries {
			            player: {
			                id,
			                name
			            },
			            timestamp_milliseconds := (select duration_get(<cal::relative_duration>.time, "milliseconds")),
			            formatted_time := (select to_str(.time, "FMHH24:MI:SS.MS"))
			        } filter .status = Status.Approved  order by .time limit 1)
			    } order by .placement offset <int32>$offset limit <int32>$limit
			),
			count := (select count(Level))
		}
	`, {
		offset: (req.page - 1) * limit,
		limit
	}));
});

app.get('/api/player/:player', async (req, res) => {
	const playerName = req.params.player;
	const isId = /[a-zA-Z0-9\-]+/.test(playerName) && playerName.replaceAll("-", "").length == 32;

	const player = await db.querySingle(`
	    select Player {
	        id,
	        name,
	        points,
	        verifications: {
	            level := (
	                with ID := .id
	                select Level {
	                    name,
	                    level_id,
	                    placement
	                } filter .id = ID
	            ),
	            video_id
	        },
	        records := (select .entries {
	            level: { name, level_id, placement },
	            timestamp_milliseconds := (select duration_get(<cal::relative_duration>.time, "milliseconds")),
	            formatted_time := (select to_str(.time, "FMHH24:MI:SS.MS")),
	            video_id,
	            mobile,
	            rank
	        } order by .level.placement),
	        rank,
	        device
	    } filter .name = ((<Player><uuid><str>$playerName).name if <bool>$isId else <str>$playerName)
	`, {
		playerName,
	    isId
	});

	if (!player) {
		res.status(400).json({
			error: true,
			code: 'bad_user'
		});
	} else {
		res.json(player);
	}
});
