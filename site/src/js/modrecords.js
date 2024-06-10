const records = document.querySelectorAll('.record')

records.forEach(r => {
    const selectable = !!r.querySelector('.video .select')
    const iframe = r.querySelector('.video .preview iframe')
    const img = r.querySelector('.video .preview img')
    const link = r.querySelector('.video .preview a')
    const video = r.querySelector('.video .select :first-child')
    const raw = r.querySelector('.video .select :last-child')

    img.addEventListener('click', e => {
        iframe.src = `https://www.youtube-nocookie.com/embed/${getVideoIDFromURL(e.target.src)}/`
        e.target.style.display = 'none'
        iframe.style.display = ''
    })

    ;(function () {
        if (!selectable) return

        const videourl = video.attributes['data-url'].value
        const rawid = getVideoIDFromURL(raw.attributes['data-url'].value)
        const rawurl = rawid ?
            `https://www.youtube-nocookie.com/embed/${rawid}/`
            : raw.attributes['data-url'].value
        const imgurl = `https://i1.ytimg.com/vi/${getVideoIDFromURL(videourl)}/hqdefault.jpg`
        const rawimg = `https://i1.ytimg.com/vi/${rawid}/hqdefault.jpg`

        link.style.display = 'none'
        iframe.style.display = 'none'

        iframe.src = ''

        video.addEventListener('click', e => {
            if (e.target.className == 'selected') return

            video.className = 'selected'
            raw.className = ''

            if (iframe.style.display == '') {
                iframe.style.display = ''
                link.style.display = 'none'
                img.style.display = 'none'

                iframe.src = videourl

                return
            }
            
            iframe.style.display = 'none'
            link.style.display = 'none'
            img.style.display = ''

            iframe.src = ''
            img.src = imgurl
        })

        raw.addEventListener('click', e => {
            if (e.target.className == 'selected') return

            video.className = ''
            raw.className = 'selected'

            if (rawid) {

                if (iframe.style.display == '') {
                    iframe.style.display = ''
                    link.style.display = 'none'
                    img.style.display = 'none'
    
                    iframe.src = rawurl
    
                    return
                }

                iframe.style.display = 'none'
                link.style.display = 'none'
                img.style.display = ''

                iframe.src = ''
                img.src = rawimg

                return
            }

            iframe.style.display = 'none'
            link.style.display = ''
            img.style.display = 'none'

            iframe.src = ''
        })
    })()
})

document.querySelectorAll('.submit select').forEach(s => s.addEventListener('change', e => {
    const submit = e.target.parentElement.lastElementChild
    const selected = e.target.querySelector('option[selected=""]').value
    const denied = e.target.value == 'denied'
    const reason = e.target.parentElement.querySelector('input[name="reason"]')

    submit.disabled = selected == e.target.value
    reason.type = denied ? 'text' : 'hidden'
}))

let scrollTimeout = false

document.body.addEventListener('scroll', () => {
    if (scrollTimeout) return

    records.forEach(r => {
        const iframe = r.firstElementChild.lastElementChild.lastElementChild.querySelector('iframe')
        if (iframe.style.display == 'none') return

        const rect = iframe.getBoundingClientRect()
        if (rect.bottom > 0 && rect.top < innerHeight) return
        
        iframe.style.display = 'none'
        iframe.nextElementSibling.style.display = ''
        iframe.src = ''
    })

    scrollTimeout = true
    setTimeout(() => scrollTimeout = false, 200)
}, { passive: true })