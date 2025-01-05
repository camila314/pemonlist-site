import app from '../app';

const render_static = (page) => app.get(`/${page}`, (req, res) => {
	res.render(page);
});

render_static('rules');
render_static('terms');
render_static('privacy');
render_static('credits');
render_static('oauth');
