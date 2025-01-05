import {Client} from 'edgedb';
import {createClient} from 'edgedb';
export * as schema from '../dbschema/interfaces';

export const db = createClient();

export class EdgeCache {
    private db: Client;
    private querys: Record<string, any>;
    private query_singles: Record<string, any>;

    constructor(db: Client, interval: number) {
        this.db = db;
        this.querys = {};
        this.query_singles = {};

        setInterval(() => {
            this.clear();
        }, interval);
    }

    async query<T>(query: string, args: Record<string, any> = {}): Promise<any> {
        const key = query + JSON.stringify(
            Object.keys(args)
                .sort()
                .map((k) => args[k])
        );

        if (this.querys[key]) {
            return <T>this.querys[key];
        }

        const promise = await this.db.query<T>(query, args);
        this.querys[key] = promise;
        return promise;
    }

    async query_single<T>(query: string, args: Record<string, any> = {}): Promise<any> {
        const key = query + JSON.stringify(
            Object.keys(args)
                .sort()
                .map((k) => args[k])
        );

        if (this.query_singles[key]) {
            return <T>this.query_singles[key];
        }

        const promise = await this.db.query<T>(query, args);
        this.query_singles[key] = promise;
        return promise;
    }

    clear(): void {
        this.querys = {};
        this.query_singles = {};
    }
}


/*export const accountCache = new EdgeCache(db, 1000 * 60 * 5);
export const playerCache = new EdgeCache(db, 1000 * 60 * 5);*/

export default db;