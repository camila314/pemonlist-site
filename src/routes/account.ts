import app from '../app';
import db from '../db';
import { RequestHandler } from 'express';
import { getDiscordAuth } from './discord';

export const requireLogin: RequestHandler = async (req, res, next) => {
	if (!req.account) {
		return res.redirect('/login');
	}
	next();
}

app.get('/login', async (req, res) => {
	if (req.account)
		return res.redirect('/account');

	res.render('login');
});

app.get('/account', requireLogin, async (req, res) => {
	if (req.account!.status === "None")
		return res.redirect('/account/setup');

	const migration = await db.querySingle(`
		select MigrationRequest {
		    id,
		    requested := (select to_str(.created_at, "FMDD FMMonth, HH24:MI")),
		    discord: { global_name, user_id, username, avatar, accent_color, banner }
		} filter .account.id = <uuid><str>$id limit 1
	`, {id: req.account!.id});

	res.render('account/account', {migration});
});
app.post('/account', requireLogin, async (req, res) => {
	if (req.body.method === "deleterecord") {
		await db.execute(`delete <Entry><uuid><str>$entry_id`, { entry_id: req.body.id });
	}
	res.redirect('/account');
})

app.get('/account/migrate', requireLogin, async (req, res) => {
	if (req.account!.status !== "None")
		return res.redirect('/account');

	const players = await db.query(`select Player { name } order by .name`);
	res.render('account/migrate', {players});
});
app.post('/account/migrate', requireLogin, async (req, res) => {
	let discord;
	if (req.body.discord) {
		const redirect = `${req.protocol}://${req.headers.host}/api/auth/discord`;
		discord = await getDiscordAuth(redirect, req.body.discord);

		if (!discord)
			return res.redirect('/account/migrate');
	}

	await db.execute(`
		insert MigrationRequest {
		    discord := ((insert Discord {
		    	user_id := <str>$user_id,
		    	username := <str>$discord_username,
		    	global_name := <str>$global_name,
		    	avatar := <str>$avatar,
		    	accent_color := <str>$accent_color,
		    	banner := <str>$banner
		    }) if <bool>$has_discord else (<Discord><uuid><str>$prior_discord)),

		    account := <Account><uuid><str>$account_id,
		    player := (select Player filter .name = <str>$username)
		};

		update <Account><uuid><str>$account_id set {
		    status := AccountStatus.Migrating
		};

		update Player filter .name = <str>$username set {
		    device := <Device><str>$device
		}
	`, {
		account_id: req.account!.id,
		username: req.body.username,
		device: req.body.device.replace(/(.)(?=.+)/, a=>a.toUpperCase()),
		prior_discord: req.account!.discord?.id,
		has_discord: discord !== undefined,
		user_id: discord?.user_id ?? "",

		discord_username: discord?.username ?? "",
		global_name: discord?.global_name ?? "",
		avatar: discord?.avatar ?? "",
		accent_color: discord?.accent_color ?? "",
		banner: discord?.banner ?? ""
	});

	res.redirect('/account');
});


app.get('/account/settings', requireLogin, async (req, res) => {
	if (req.account!.status !== "Done")
		return res.redirect('/account');

	res.render('account/settings');
});
app.post('/account/settings', requireLogin, async (req, res) => {
	switch (req.body.method) {
		case "logout":
			await db.execute(`delete AuthToken filter .token = <str>$token`, { token: req.cookies.token });
			res.redirect('/');
			break;

		case "delete":
			await db.execute(`
				delete AuthToken filter .account.id = <uuid><str>$account_id;
				delete MigrationRequest filter .account.id = <uuid><str>$account_id;
				delete Account filter .id = <uuid><str>$account_id;
			`, { account_id: req.account!.id });
			res.redirect('/');
			break;

		case "update":
			const name = ((req.body.name || "") as string).trim();
			if (name.length == 0 || name.length > 25)
				return res.redirect('/account/settings');

			await db.execute(`
                update Player filter .id = <uuid><str>$player_id set {
                    name := <str>$username,
                    device := <Device><str>$device
                };
                update Account filter .id = <uuid><str>$account_id set {
                    profile_shape := <ProfileShape><str>$profile_shape
                }
			`, {
				account_id: req.account!.id,
				player_id: req.account!.player!.id,
				username: name,
				device: req.body.device.replace(/(.)(?=.+)/, a=>a.toUpperCase()),
				profile_shape: req.body.profileshape.replace(/(.)(?=.+)/, a=>a.toUpperCase())
			});
			res.redirect('/account/settings');
			break;

		default:
			res.redirect('/account/settings');
			break;
	}
});

app.get('/account/setup', requireLogin, async (req, res) => {
	if (req.account!.status !== "None")
		return res.redirect('/account');

	res.render('account/setup');
});
app.post('/account/setup', requireLogin, async (req, res) => {
	console.log("WHAT");
	const name = ((req.body.username || "") as string).trim();
	if (name.length == 0 || name.length > 25) {
		console.log("what...", name);
		return res.redirect('/account/setup');
	}

	const goSetup = await db.querySingle(`
		select true if exists (select Player { id } filter .name = <str>$username) else (select (
			false,
            (update Account filter .id = <uuid><str>$account_id set {
                status := AccountStatus.Done,
                player := (insert Player {
		            name := <str>$username,
		            device := <Device><str>$device
		        })
            })
		).0)
	`, {
		account_id: req.account!.id,
		username: name,
		device: req.body.device.replace(/(.)(?=.+)/, a=>a.toUpperCase())
	});

	res.redirect(goSetup ? `/account/migrate?username=${name}&device=${req.body.device}` : '/account');
});
