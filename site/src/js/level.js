const main = document.querySelector('main')

;(async function() {
    const image = await fetch(`https://raw.githubusercontent.com/cdc-sys/level-thumbnails/main/thumbs/${location.pathname.match(/(\d+)/)[0]}.png`)

    if (image.status == 200) {
        const loader = new Image()
        loader.src = image.url
        await new Promise(resolve => loader.onload = resolve)
        main.classList.add('transparent')
        main.style.background = `url('${image.url}')`
        console.log('bg loaded')
    }
})()

function adjustParallax() {
    main.style.backgroundPositionY = `${document.body.scrollTop * 0.3}px`
}

document.body.addEventListener('scroll', adjustParallax, { passive: true })

adjustParallax()