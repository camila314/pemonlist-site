const selectable = !!document.querySelector('.video .select')
const iframe = document.querySelector('.video .preview iframe')
const link = document.querySelector('.video .preview a')
const video = document.querySelector('.video .select :first-child')
const raw = document.querySelector('.video .select :last-child')

;(function () {
    if (!selectable) return

    const videourl = video.attributes['data-url'].value
    const rawid = getVideoIDFromURL(raw.attributes['data-url'].value)
    const rawurl = rawid ?
        `https://www.youtube-nocookie.com/embed/${rawid}/`
        : raw.attributes['data-url'].value

    link.style.display = 'none'
    iframe.src = videourl
    link.href = rawurl

    video.addEventListener('click', e => {
        if (e.target.className == 'selected') return

        video.className = 'selected'
        raw.className = ''

        if (rawid) return iframe.src = videourl
        iframe.style.display = ''
        link.style.display = 'none'
    })

    raw.addEventListener('click', e => {
        if (e.target.className == 'selected') return

        video.className = ''
        raw.className = 'selected'

        if (rawid) return iframe.src = rawurl
        iframe.style.display = 'none'
        link.style.display = ''
    })
})()

document.querySelectorAll('.submit select').forEach(s => s.addEventListener('change', e => {
    const submit = e.target.parentElement.children[1]
    const selected = e.target.querySelector('option[selected=""]').value

    submit.disabled = selected == e.target.value
}))