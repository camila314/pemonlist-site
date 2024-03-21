const query = window.matchMedia('(max-width: 1012px)')
query.addEventListener('change', () => {
    document.querySelector('nav button.overflow').classList.remove('enabled')
    document.querySelector('nav span.overflow').classList.remove('enabled')
})

document.addEventListener('click', e => {
    if (document.querySelector('nav').contains(e.target)) return

    document.querySelector('nav button.overflow').classList.remove('enabled')
    document.querySelector('nav span.overflow').classList.remove('enabled')
})

document.querySelector('nav button.overflow').addEventListener('click', () => {
    const enabled = document.querySelector('nav span.overflow').classList.toggle('enabled')
    document.querySelector('nav button.overflow').classList.toggle('enabled', enabled)
})

if ("ontouchstart" in window) document.body.classList.add('touch')