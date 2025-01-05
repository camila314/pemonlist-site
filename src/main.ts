import { app } from './app';

import './routes/static';
import './routes/account';
import './routes/mod';
import './routes/index';
import './routes/submit';
import './routes/discord';
import './routes/google';
import './routes/api';

app.get('*fallback', (req, res) => {
    res.status(404).render('fallback', {status: '404: Page does not exist'});
})

app.use((err, req, res, next) => {
    res.status(500).render('fallback', {status: `500: ${err}`});
});

app.listen(8111, () => {
    console.log(`Example app listening at http://localhost:8111`);
});
