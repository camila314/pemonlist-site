class Logger {
    prefix = ''
    
    constructor(...prefixes) {
        prefixes.map(p => p.split(' ').map(s => (s[0].toUpperCase() + s.slice(1))).join(' '))
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
    mainLogger.info('Device seems to be a touchscreen')
    document.body.classList.add('touch')
}

const mobileDevices = [ 'android', 'webos', 'iphone', 'ipad', 'ipod', 'blackberry', 'windows phone' ]
let mobile = false

if (!navigator.userAgentData) {
    mainLogger.warn('navigator.userAgentData does not exist, checking against navigator.userAgent')

    mobileDevices.forEach(d => {
        let m = navigator.userAgent.match(new RegExp(d, 'i')) != null
        if (m) return mobile = true
    })
} else mobile = navigator.userAgentData.mobile

mainLogger.info(`Browser reporting device as a ${mobile ? 'mobile' : 'desktop'} device`)