const jumpLogger = new Logger('Jump')

const jumpButton = document.createElement('button')
jumpButton.classList.value = 'jump hidden'

const icon = document.createElement('span')
icon.className = 'material-symbols-outlined'
icon.innerText = 'arrow_upward_alt'

const text = document.createElement('p')
text.innerText = 'Scroll to Top'

jumpButton.append(icon, text)
document.body.appendChild(jumpButton)

;(async function() {
    if (!jumpButton) return

    const worthJumping = document.body.scrollHeight > window.innerHeight * 2.5

    if (!worthJumping) return jumpLogger.log('Page too short to enable jump button')

    const main = document.querySelector('main')

    main.style.paddingBottom = '72px' // [52px button height] + [10px padding top and bottom]

    jumpButton.addEventListener('click', () => document.body.scrollTo({ top: 0, behavior: 'smooth' }))

    let contentHeight = main.clientHeight + 70 + (100 * !cookies.agree)
    let footerHeight = main.clientHeight + 8 + (100 * !cookies.agree) // 70 - [52px button height] - [10px padding]

    let lastClientHeight = 0

    // snap to proper place after window it resized
    window.addEventListener('resize', () => {
        if (main.clientHeight == lastClientHeight) return

        contentHeight = main.clientHeight + 70 + (100 * !cookies.agree)
        footerHeight = main.clientHeight + 8 + (100 * !cookies.agree)
        updateJumpButton()

        lastClientHeight = main.clientHeight
    }, { passive: true })

    const shownHeight = window.innerHeight * .7

    function updateJumpButton() {
        const shown = document.body.scrollTop > shownHeight
        jumpButton.classList.toggle('hidden', !shown)

        const tooLow = contentHeight < document.body.scrollTop + innerHeight
        jumpButton.classList.toggle('low', tooLow)
        jumpButton.style.top = tooLow ? (footerHeight + 'px') : 'unset'
    }

    document.body.addEventListener('scroll', updateJumpButton, { passive: true })

    updateJumpButton()
})()