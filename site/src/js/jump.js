const jumpLogger = new Logger('Jump')

const jumpButton = document.querySelector('.jump')
const body = document.body

;(async function() {
    if (!jumpButton) return

    const worthJumping = body.scrollHeight > window.innerHeight * 2.5

    if (!worthJumping) return jumpLogger.log('Page too short to enable jump button')

    const main = document.querySelector('main')

    jumpButton.addEventListener('click', () => body.scrollTo({ top: 0, behavior: 'smooth' }))

    const contentHeight = main.clientHeight + 70
    const footerHeight = main.clientHeight + 8 // i don't know why but it works so shut up

    const shownHeight = window.innerHeight * .3

    function updateJumpButton() {
        const shown = body.scrollTop > shownHeight
        jumpButton.classList.toggle('hidden', !shown)

        const toLow = contentHeight < body.scrollTop + innerHeight
        jumpButton.classList.toggle('low', toLow)
        jumpButton.style.top = toLow ? (footerHeight + 'px') : 'unset'
    }

    body.addEventListener('scroll', updateJumpButton, { passive: true })

    updateJumpButton()
})()