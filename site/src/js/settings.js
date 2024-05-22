const deletebtn = document.querySelector('.delete')
const profileshape = document.querySelector('.profile .select select')
const img = document.querySelector('.profile .img img')

profileshape.addEventListener('change', e => {
    img.className = e.target.value
})

deletebtn.addEventListener('click', async e => {
    e.preventDefault()

    const cover = document.createElement('div')
    cover.className = 'cover'

    const confirmation = document.createElement('div')
    confirmation.className = 'confirmation'

    const header = document.createElement('h1')
    header.innerText = 'Confirmation'

    const body = document.createElement('p')
    body.innerText = 'Are you sure you want to delete your account?\nIt will be lost for a very long time!'

    const disclaimer = document.createElement('p')
    disclaimer.innerText = 'All current records will be kept.'
    disclaimer.className = 'disclaimer'

    const input = document.createElement('span')

    const yes = document.createElement('button')
    yes.innerText = 'I\'m sure'
    yes.className = 'yes'

    const no = document.createElement('button')
    no.innerText = 'Nevermind'

    input.append(yes, no)
    body.appendChild(disclaimer)
    confirmation.append(header, body, input)
    cover.appendChild(confirmation)
    document.body.appendChild(cover)

    await frame()

    async function hide(e) {
        if (e.target != cover && e.target != no) return

        cover.classList.remove('visible')
        confirmation.classList.remove('visible')

        await ms(200)

        cover.remove()
    }

    cover.addEventListener('click', hide)
    no.addEventListener('click', hide)
    yes.addEventListener('click', () => {
        deletebtn.parentElement.submit()
    })

    cover.classList.add('visible')
    confirmation.classList.add('visible')
})