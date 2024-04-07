class Logger {
    prefix = ''
    constructor(prefix = null) {
        if (!prefix) return

        this.prefix = `[${
            prefix.split(' ').map(s => (s[0].toUpperCase() + s.substr(1))).join(' ')
        }]`
    }

    debug(...content) {
        console.debug(`%c ${this.prefix}`, 'color:rgb(175,0,175)', ...content)
    }

    error(...content) {
        console.error(`${this.prefix}`, ...content)
    }

    info(...content) {
        console.info(`%c ${this.prefix}`, 'color:rgb(175,0,175)', ...content)
    }

    log(...content) {
        console.log(`%c ${this.prefix}`, 'color:rgb(175,0,175)', ...content)
    }

    warn(...content) {
        console.warn(`${this.prefix}`, ...content)
    }

    table(...content) {
        console.table(`%c ${this.prefix}`, 'color:rgb(175,0,175)', ...content)
    }
}

const mainLogger = new Logger('Main')

mainLogger.log('suck it mechabrandon')

if ('ontouchstart' in window) document.body.classList.add('touch')