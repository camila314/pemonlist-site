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
        console.table(...content)
    }
    
    group(...content) {
        console.group(`%c ${this.prefix}`, 'color:rgb(175,0,175)', ...content)
    }
    
    groupCollapsed(...content) {
        console.groupCollapsed(`%c ${this.prefix}`, 'color:rgb(175,0,175)', ...content)
    }

    groupEnd() {
        console.groupEnd()
    }
}

const mainLogger = new Logger('Main')

// mainLogger.log('suck it mechabrandon')

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

function getVideoIDFromURL(url) {
    let parsedURL = new URL(url)

    // converting other URLs
    if (parsedURL.pathname == '/oembed')
        parsedURL = new URL(parsedURL.searchParams.get('url'))

    if (parsedURL.pathname == '/attribution_link')
        parsedURL = new URL(parsedURL.searchParams.get('u'), 'https://youtube.com')


    // youtube.com/watch?v=dQw4w9WgXcQ
    if (parsedURL.searchParams.get('v')) {
        return parsedURL.searchParams.get('v')
    }

    // youtube.com/?vi=dQw4w9WgXcQ
    if (parsedURL.searchParams.get('vi')) {
        return parsedURL.searchParams.get('vi')
    }

    // checks for URL slugs like youtube.com/watch/dQw4w9WgXcQ or youtube.com/v/dQw4w9WgXcQ
    for (const slug of [ 'embed', 'e', 'shorts', 'live', 'watch', 'v', 'vi' ]) {
        const match = parsedURL.pathname.match(new RegExp(`/${slug}/(.+)$`))
        if (match) return match[1]
            .split('&')[0] // protects `vi` from returning feature parameter
            .replace(/^\/?([^\/]+)\/?.*$/, '$1') // removes preceding or trailing slug(s)
    }

    // youtube.com/user/GitHub#p/a/u/1/lalOy8Mbfdc
    if (parsedURL.hash.match(/#p\/(?:a\/)?u\/\d+\/.+$/)) {
        return parsedURL.hash.match(/#p\/(?:a\/)?u\/\d+\/(.+)$/)[1]
            .split('?')[0] // protects from returning rel parameter
            .replace(/^\/?([^\/]+)\/?.*$/, '$1') // removes preceding or trailing slug(s)
    }

    // youtu.be/dQw4w9WgXcQ
    if (parsedURL.hostname.match(/youtu\.be/)) {
        return parsedURL.pathname.slice(1)
            .split('&')[0] // protects from returning feature parameter
            .replace(/^\/?([^\/]+)\/?.*$/, '$1') // removes preceding or trailing slug(s)
    }
}

const ms = async (ms) => await new Promise(r => setTimeout(r, ms))
const frame = async () => await new Promise(r => requestAnimationFrame(r))

const cookies = {}
document.cookie.split('; ').forEach(c => cookies[c.split('=')[0]] = decodeURIComponent(c.split('=')[1]))