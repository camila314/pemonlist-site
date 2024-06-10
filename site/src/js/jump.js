const jumpLogger = new Logger('Jump')

const jumpButton = document.createElement('button')
jumpButton.classList.value = 'jump hidden'

const icon = document.createElement('span')
icon.className = 'material-symbols-outlined'
icon.innerText = 'arrow_upward_alt'

const text = document.createElement('p')
text.innerText = 'Scroll to Top'

const body = document.body

jumpButton.append(icon, text)
body.appendChild(jumpButton)

;(async function() {
    if (!jumpButton) return

    const worthJumping = body.scrollHeight > window.innerHeight * 2.5

    if (!worthJumping) return jumpLogger.log('Page too short to enable jump button')

    const main = document.querySelector('main')

    main.style.paddingBottom = '72px' // [52px button height] + [10px padding top and bottom]

    jumpButton.addEventListener('click', () => body.scrollTo({ top: 0, behavior: 'smooth' }))

    let contentHeight = main.clientHeight + 70
    let footerHeight = main.clientHeight + 8 // 70 - [52px button height] - [10px padding]

    let lastClientHeight = 0

    // snap to proper place after window it resized
    window.addEventListener('resize', () => {
        if (main.clientHeight == lastClientHeight) return

        contentHeight = main.clientHeight + 70
        footerHeight = main.clientHeight + 8
        updateJumpButton()

        lastClientHeight = main.clientHeight
    }, { passive: true })

    const shownHeight = window.innerHeight * .7

    function updateJumpButton() {
        const shown = body.scrollTop > shownHeight
        jumpButton.classList.toggle('hidden', !shown)

        const tooLow = contentHeight < body.scrollTop + innerHeight
        jumpButton.classList.toggle('low', tooLow)
        jumpButton.style.top = tooLow ? (footerHeight + 'px') : 'unset'
    }

    body.addEventListener('scroll', updateJumpButton, { passive: true })

    updateJumpButton()
})()