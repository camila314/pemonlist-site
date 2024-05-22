const id = document.querySelector('.info .id p')

if (id != null) id.addEventListener('click', async e => {
    const exists = document.querySelector('.copied')

    if (exists) exists.remove()

    const copied = document.createElement('div')
    copied.classList.value = 'copied'
    copied.innerText = 'Copied!'

    await navigator.clipboard.writeText(id.firstChild.data)

    document.querySelector('.info .id').appendChild(copied)
    await frame()
    copied.classList.toggle('open')

    await ms(2200)

    copied.style.transitionTimingFunction = 'cubic-bezier(0.55, 0.06, 0.68, 0.19)'
    await frame()
    copied.classList.toggle('open')

    await ms(200)

    copied.remove()
})