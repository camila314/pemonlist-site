import express from 'express';

import nunjucks from 'nunjucks';
import cookieParser from "cookie-parser";

import translations from './translations';
import { db } from './db';

import { AuthToken, Account } from "../dbschema/interfaces";

export const app = express();

const nj = nunjucks.configure('www/html', {
	autoescape: true
}).addFilter('sandwich', function(str, count) {
    return str.split("{}")[count];
});

app.use('/static', express.static('www/static'));
app.use('/translations', express.static('www/translations'));
app.use(cookieParser());
app.use(express.urlencoded({ extended: false }));

declare module "express-serve-static-core" {
  interface Request {
    account?: Account;
    page: number;
    timestamp: number;
  }
}

app.use(async (req, res, next) => {
	req.page = parseInt(req.query.page as string) || 1;
	req.timestamp = Date.now();
	const token = req.cookies.token;

	if (token) {
		req.account = await db.querySingle<AuthToken>(`
			select AuthToken {
			    account: {
			        id,
			        image,
			        profile_shape,
			        status,
			        mod,
			        player: {
			            id, 
			            name,
			            points,
			            verifications: { name, level_id, placement, video_id },
			            records := (select .entries {
			                level: { name, level_id, placement },
			                time_format := (select to_str(.time, "FMHH24:MI:SS")),
			                time_ms := (select to_str(.time, "MS")),
			                video_id,
			                rank
			            } order by .level.placement),
			            unverified_records := (select .unverified_entries {
			                id,
			                level: { name, level_id, placement },
			                time_format := (select to_str(.time, "FMHH24:MI:SS")),
			                time_ms := (select to_str(.time, "MS")),
			                video_id,
			                status,
			                reason
			            } order by .level.placement),
			            rank,
			            device
			        },
			        youtube,
			        discord
			    }
			} filter .token = <str>$token and .expires > <datetime>datetime_of_statement() limit 1
		`, {token}).then(res => res?.account);
	}

	res.render = async function(template, data = {}) {
		const translation = translations[req.cookies.lang] || translations["en"];
		if (req.account) {
			data["account"] = req.account;
		}

		this.send(nj.render(`${template}.html`, {translation, ...data}));
	};

	next();
});

export default app;