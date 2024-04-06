const main = document.querySelector('main')

;(async function() {
    const image = await fetch(`https://raw.githubusercontent.com/cdc-sys/level-thumbnails/main/thumbs/${location.pathname.match(/(\d+)/)[0]}.png`)

    if (image.status == 200) {
        main.classList.add('transparent')
        main.style.background = `url('${image.url}')`
    }
})()

function adjustParallax(event) {
    main.style.backgroundPositionY = `${event.target.scrollTop * 0.3}px`
}

document.body.addEventListener('scroll', adjustParallax)

adjustParallax()