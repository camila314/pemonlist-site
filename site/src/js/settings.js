const username = document.getElementById('name')
const deletebtn = document.querySelector('.delete')
const profileshape = document.querySelector('.profile .select select')
const img = document.querySelector('.profile .img img')

username.addEventListener('input', e => {
    e.target.setCustomValidity('')
    
    if (e.target.value.trim().length > 25) return e.target.setCustomValidity('Your username must be 25 characters or less.\nPreceding or trailing whitespace will be trimmed.')
})

profileshape.addEventListener('change', e => {
    img.className = e.target.value
})

deletebtn.addEventListener('click', async e => {
    e.preventDefault()

    const cover = document.createElement('div')
    cover.className = 'cover'

    const modal = document.createElement('div')
    modal.className = 'modal'

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
    yes.className = 'catastrophic'

    const no = document.createElement('button')
    no.innerText = 'Nevermind'

    input.append(yes, no)
    body.appendChild(disclaimer)
    modal.append(header, body, input)
    cover.appendChild(modal)
    document.body.appendChild(cover)

    await frame()

    async function hide(e) {
        if (e.target != cover && e.target != no) return

        cover.classList.remove('visible')
        modal.classList.remove('visible')

        await ms(200)

        cover.remove()
    }

    cover.addEventListener('click', hide)
    no.addEventListener('click', hide)
    yes.addEventListener('click', async () => {
        deletebtn.parentElement.submit()
    })

    cover.classList.add('visible')
    modal.classList.add('visible')
})