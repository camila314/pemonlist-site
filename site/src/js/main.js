class Logger {
    prefix = ''
    
    constructor(...prefixes) {
        prefixes.map(p => p.split(' ').map(s => (s[0].toUpperCase() + s.substr(1))).join(' '))
        this.prefix = `[${prefixes.join('] [')}]`
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

if ('ontouchstart' in window) {
    mainLogger.info('device seems to be a touchscreen')
    document.body.classList.add('touch')
}