const jumpLogger = new Logger('Jump')

const jumpButton = document.querySelector('.jump')
const body = document.body

;(async function() {
    if (!jumpButton) return

    const worthJumping = body.scrollHeight > window.innerHeight * 2

    if (!worthJumping) return jumpLogger.log('Page too short to enable jump button')

    jumpButton.addEventListener('click', () => body.scrollTo({ top: 0, behavior: 'smooth' }))

    function updateJumpButton() {
        const shown = body.scrollTop > 120
        jumpButton.classList.toggle('hidden', !shown)
    }

    body.addEventListener('scroll', updateJumpButton, { passive: true })

    updateJumpButton()
})()