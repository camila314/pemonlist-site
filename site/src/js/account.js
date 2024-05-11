const id = document.querySelector('.info .id p')

id.addEventListener('click', async e => {
    const bounding = id.getBoundingClientRect()

    const copied = document.createElement('div')

    copied.className = 'copied'
    copied.style.bottom = bounding.y - 3 + 'px'
    copied.style.left = bounding.x + (bounding.width / 2) + 'px'

    document.body.appendChild(copied)
})