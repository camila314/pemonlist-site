let url = document.querySelectorAll('.video input')[0]
const container = document.querySelector('.video').children[1]
const iframe = document.querySelector('.preview iframe')
const videoid = document.querySelector('#videoid')
const time = document.getElementById('time')
const timeplain = document.getElementById('timeplain')
const submit = document.querySelector('input[type="submit"]')
const level = document.getElementById('level')

// unit tests for YouTube URLs

let testLogger
function testURLs(urls) {
    const units = urls.split(/\n{2,}/)

    testLogger.info(`Testing ${units.length + 1} units`)

    for (let unitnum in units) {
        const unit = units[unitnum].split('\n')

        for (let test of unit) {
            let id = false

            try { id = getVideoIDFromURL(test) }
            catch (e) {
                return testLogger.error(`Unit ${parseInt(unitnum) + 1} failed on test ${test}\nRan into error ${e}`)
            }

            if (!id)
                return testLogger.error(`Unit ${parseInt(unitnum) + 1} failed on test ${test}\nOutput was \`${id}\``)
        }

        testLogger.groupCollapsed(`Unit ${parseInt(unitnum) + 1} passed`)
        testLogger.table(unit)
        testLogger.groupEnd()
    }

    return true
}

// ;(async function() {
//     testLogger = new Logger('Submit', 'Tests', 'ActiveYouTubeURLFormats')
//     const ActiveYouTubeURLFormats = await fetch('https://gist.githubusercontent.com/rodrigoborgesdeoliveira/987683cfbfcc8d800192da1e73adc486/raw/ee0e9f38e519295f7c62b827004948104b9962e8/ActiveYouTubeURLFormats.txt')
//         .then(a => a.text())
//     if (!testURLs(ActiveYouTubeURLFormats)) return

//     testLogger = new Logger('Submit', 'Tests', 'DeprecatedYouTubeURLFormats')
//     const DeprecatedYouTubeURLFormats = await fetch('https://gist.githubusercontent.com/rodrigoborgesdeoliveira/987683cfbfcc8d800192da1e73adc486/raw/ee0e9f38e519295f7c62b827004948104b9962e8/DeprecatedYouTubeURLFormats.txt')
//         .then(a => a.text())
//     if (!testURLs(DeprecatedYouTubeURLFormats.replace(/\n\n/g, '\n'))) return

//     new Logger('Submit', 'Tests').info('All tests passed!')
// })()

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
        if (match) return match[1].split('&')[0] // split protects `vi` from returning feature parameter
    }

    // youtube.com/user/GitHub#p/a/u/1/lalOy8Mbfdc
    if (parsedURL.hash.match(/#p\/(?:a\/)?u\/\d+\/.+$/)) {
        return parsedURL.hash.match(/#p\/(?:a\/)?u\/\d+\/(.+)$/)[1].split('?')[0] // split protects from returning rel parameter
    }

    // youtu.be/dQw4w9WgXcQ
    if (parsedURL.hostname.match(/youtu\.be/)) {
        return parsedURL.pathname.slice(1).split('&')[0] // split protects from returning feature parameter
    }
}

const submitLogger = new Logger('Submit')

document.querySelector('.beside span:first-of-type').addEventListener('click', () => {
    time.focus()
})

// https://stackoverflow.com/a/72129181
time.addEventListener('paste', function (e) {
    e.preventDefault()

    const text = e.clipboardData
        ? (e.originalEvent || e).clipboardData.getData('text/plain')
        :
        window.clipboardData
        ? window.clipboardData.getData('Text')
        : ''

    if (document.queryCommandSupported('insertText')) {
        document.execCommand('insertText', false, text)
    } else {
        const range = document.getSelection().getRangeAt(0)
        range.deleteContents()

        const textNode = document.createTextNode(text)
        range.insertNode(textNode)
        range.selectNodeContents(textNode)
        range.collapse(false)

        const selection = window.getSelection()
        selection.removeAllRanges()
        selection.addRange(range)
    }
})

time.addEventListener('input', e => {
    timeplain.setCustomValidity('')

    timeplain.value = e.target.innerText

    if (e.target.innerText.length == 0) return

    let time = e.target.innerText.split(':')
    time.push(...time.pop().split('.'))

    let valid = true
    time.forEach((e, i) => {
        let test = /^\d+$/.test(e)
        if (!test) return valid = false
        time[i] = parseInt(e)
        if (time[i] < 0) return valid = false
    })

    if (!valid) return timeplain.setCustomValidity('Please enter a valid time format.')

    if ([...time].reverse()[0].length < 3) return timeplain.setCustomValidity('Please add milliseconds.')

    time.map((e, i) => {
        if (e == 0) time.shift()
    })

    console.log(time)
})

level.addEventListener('change', e => {
    e.target.setCustomValidity('')

    const option = document.querySelector(`option[value="${e.target.value}"]`)
    if (!option) return e.target.setCustomValidity('Please input a valid level name.')
    
    document.getElementById('levelid').value = option.attributes['attr-level-id'].value
})

let processingTimeout = false
let lastID = ''

async function input(e) {
    const previewstate = document.querySelector('.select .selected').innerText.toLowerCase()

    if (e.target.id != previewstate) return

    if (e.target.value == '') 
        return container.classList.value = 'default'
    if (processingTimeout) return

    e.target.setCustomValidity('')

    if (!e.target.checkValidity() || !getVideoIDFromURL(e.target.value)) {
        if (previewstate == 'video') e.target.setCustomValidity('Please enter a valid YouTube URL.')
        videoid.value = ''
        return container.classList.value = 'failed'
    }

    container.classList.value = 'processing'
    
    const id = getVideoIDFromURL(e.target.value)

    let thumb = await fetch(`https://www.youtube.com/oembed?url=${encodeURIComponent(`https://youtu.be/${id}`)}`, { method: 'HEAD' })
    if (thumb.status != 200) {
        if (previewstate == 'video') e.target.setCustomValidity('Please enter a valid YouTube URL.')
        videoid.value = ''
        setTimeout(() => processingTimeout = false, 500)
        return container.classList.value = 'failed'
    }

    if (id != lastID) {
        iframe.src = `https://www.youtube.com/embed/${id}/`
    }
    lastID = id
    
    videoid.value = id

    processingTimeout = true
    container.classList.value = 'display'
    setTimeout(() => processingTimeout = false, 500)
}

document.querySelectorAll('.video input[type="url"]')
    .forEach(el => el.addEventListener('input', input))

document.querySelectorAll('.video .select span')
    .forEach(el => el.addEventListener('click', e => {
        document.querySelectorAll('.video .select span').forEach(e => e.className = '')
        e.target.className = 'selected'

        url = document.querySelectorAll('.video input[type="url"]')[(e.target.innerText.toLowerCase() == 'raw') * 1]

        // reset lastID before switching
        lastID = ''

        input({ target: url })
    }))

switch (mobile) {
    case true:
        document.querySelector('#device option[value="mobile"]').selected = true
    default:
        document.querySelector('#device option[value="desktop"]').selected = true
}

submit.addEventListener('click', () => {
    const video = document.getElementById('video')
    const raw = document.getElementById('raw')

    video.setCustomValidity('')

    if (videoid.value.length == 0 && raw.value.length == 0) return video.setCustomValidity('Please input at least one video.')
    else video.removeAttribute('required')

    const rawid = getVideoIDFromURL(raw.value)
    if (rawid) videoid.value = rawid

    submit.click()
})

// TODO: Remove
// timeplain: 0:03:19.325
// level: Flame Arena I
// levelid: 98052971
// video: 
// videoid: 
// raw: https://www.youtube.com/watch?v=9YOpP9KIXUM
// device: mobile