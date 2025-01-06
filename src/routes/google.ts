import app from '../app';
import db from '../db';
import { randomBytes } from 'crypto';

import * as googleapis from 'google-auth-library';

const oauth2Client = new googleapis.OAuth2Client(
	process.env.GOOGLE_OAUTH2_CLIENT_ID,
	process.env.GOOGLE_OAUTH2_CLIENT_SECRET
);

app.get('/api/auth/google', async (req, res) => {
	const redirect = `https://${req.headers.host}/api/auth/google`;
	const authUrl = oauth2Client.generateAuthUrl({
		access_type: 'offline',
		scope: 'email',
		redirect_uri: redirect
	});

	if (!req.query.code)
		return res.redirect(authUrl);

	const token = await oauth2Client.getToken({
		code: req.query.code.toString(),
		redirect_uri: redirect
	}).catch(_ => undefined);
	if (!token)
		return res.redirect(authUrl);

	const ticket = await oauth2Client.verifyIdToken({
		idToken: token.tokens.id_token!,
		audience: process.env.GOOGLE_OAUTH2_CLIENT_ID
	}).catch(_ => undefined);
	if (!ticket)
		return res.redirect(authUrl);

	const payload = ticket.getPayload();
	if (!payload || !payload.email_verified || !payload.email)
		return res.redirect(authUrl);

	const randomToken = randomBytes(100)
		.toString('base64')
		.replace(/\+|\/|\=/g, "")
		.substring(0, 64);

	await db.execute(`
		insert AuthToken {
		    token := <str>$token,
		    account := (
				insert Account {
					email :=  <str>$email,
					oauth2 :=  <str>$oauth,
					player := <default::Player>{}
				} unless conflict on .email else (select Account)
		    )
		}
	`, {
		email: payload.email,
		oauth: req.query.code,
		token: randomToken
	});

	res.cookie('token', randomToken, {
		maxAge: 1000 * 60 * 60 * 24 * 7,
		httpOnly: true
	});

	res.redirect('/account');
});
