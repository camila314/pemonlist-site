let url = document.querySelectorAll('.video input')[0]
const container = document.querySelector('.video').children[1]
const iframe = document.querySelector('.preview iframe')
const videoid = document.querySelector('#videoid')
const time = document.getElementById('time')
const timeplain = document.getElementById('timeplain')
const submit = document.querySelector('input[type="submit"]')
const level = document.getElementById('level')

const submitLogger = new Logger('Submit')

document.querySelector('.beside span:first-of-type').addEventListener('click', () => {
    time.focus()
})

function checkValidityOfTime(time) {
    return /^(?:\d{1,2}:)?(?:\d{1,2}:)?\d{2}(?:\.\d{3})?$/.test(time)
}

time.addEventListener('input', e => {
    e.target.setCustomValidity('')

    if (!e.target.value.trim()) return

    if (!checkValidityOfTime(e.target.value)) e.target.setCustomValidity('Please enter a valid time format.')
})

level.addEventListener('change', e => {
    e.target.setCustomValidity('')

    const option = document.querySelector(`option[value="${e.target.value}"]`)
    if (!option) return e.target.setCustomValidity('Please input a valid level name.')
    
    document.getElementById('levelid').value = option.dataset.levelId
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
    if (rawid && videoid.value.length == 0) videoid.value = rawid

    submit.click()
})