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

let touch = false

if ('ontouchstart' in window) {
    mainLogger.info('Device seems to be a touchscreen')
    document.body.classList.add('touch')
    touch = true
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
    let parsedURL = null

    try { parsedURL = new URL(url) } catch { return undefined }

    if (parsedURL.hostname)

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
document.cookie.split('; ').forEach(c => {
    let value = c.split('=')[1]

    if (!isNaN(value)) value = parseFloat(value)
    else switch (value) {
        case 'true':
        case 'false':
            value = value == 'true'
            break
        default: 
            decodeURIComponent(value)
    }

    cookies[c.split('=')[0]] = value
})

cookies.add = (name, value, expires = 2147483647) => {
    document.cookie = `${name}=${encodeURIComponent(value)}; expires=${expires}; path=/`
    cookies[name] = value
}

// fix level datalist duplicates

const duplicates = []

document.querySelectorAll('datalist#levels').forEach(d => [...d.children].forEach(o => {
    let count = d.querySelectorAll(`option[value="${o.value}"]`).length
    if (count <= 1) return
    duplicates.push(o)
}))

duplicates.forEach(o => {
    o.value += ` (${o.dataset.levelId})`
    o.label = o.label.match(/^(#\d+)/)[1]
})

function createFormAndPost(action, payload = {}) {
    const form = document.createElement('form')
    form.method = 'post'
    form.action = action

    Object.keys(payload).forEach(k => {
        const input = document.createElement('input')
        input.type = 'hidden'
        input.name = k
        input.value = payload[k]

        form.appendChild(input)
    })

    document.body.appendChild(form)
    form.submit()
}

// cookies/terms warning
window.addEventListener('load', async () => {
    if (cookies.agree) return

    const warning = document.createElement('div')
    warning.className = 'warning'

    if (document.body.scrollHeight > window.innerHeight && !document.body.className.trim()) warning.classList.add('scrollbar')

    const text = document.createElement('span')
    text.innerHTML = '<p>By using this website you agree to the use of cookies, and to the <a href="/terms" class="proper">Terms & Conditions</a> and <a href="/privacy" class="proper">Privacy Policy</a>.</p><p>We will never intentionally track you or sell your data, no matter what.</p>'

    const ok = document.createElement('button')
    ok.innerText = 'Got it'

    warning.append(text, ok)
    document.body.appendChild(warning)

    ok.addEventListener('click', async () => {
        warning.classList.remove('visible')

        await ms(300)

        warning.remove()
        cookies.add('agree', true)
    })

    await frame()

    warning.classList.add('visible')

    await ms(300)

    if (document.body.scrollHeight > window.innerHeight && !document.body.className.trim()) warning.classList.add('scrollbar')
})