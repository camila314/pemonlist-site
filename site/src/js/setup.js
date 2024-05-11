const img = document.querySelector('.profile img')
const select = document.querySelector('.profile select')

select.addEventListener('change', e => img.classList.value = e.target.value)

switch (mobile) {
    case true:
        document.querySelector('#device option[value="mobile"]').selected = true
    default:
        document.querySelector('#device option[value="desktop"]').selected = true
}