const jumpButton = document.querySelector('.jump')
const body = document.body

;(async function() {
    if (!jumpButton) return

    jumpButton.addEventListener('click', () => body.scrollTo({ top: 0, behavior: 'smooth' }))

    function updateJumpButton() {
        const shown = body.scrollTop > 120
        jumpButton.classList.toggle('hidden', !shown)
    }

    body.addEventListener('scroll', updateJumpButton, { passive: true })

    updateJumpButton()
})()