const jumpLogger = new Logger('Jump')

const jumpButton = document.querySelector('.jump')
const body = document.body

;(async function() {
    if (!jumpButton) return

    const worthJumping = body.scrollHeight > window.innerHeight * 2.5

    if (!worthJumping) return jumpLogger.log('Page too short to enable jump button')

    const main = document.querySelector('main')

    main.style.paddingBottom = '72px' // [52px button height] + [10px padding top and bottom]

    jumpButton.addEventListener('click', () => body.scrollTo({ top: 0, behavior: 'smooth' }))

    const contentHeight = main.clientHeight + 70
    const footerHeight = main.clientHeight + 8 // 70 - [52px button height] - [10px padding]

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