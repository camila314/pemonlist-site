const username = document.getElementById('username')
const img = document.querySelector('.profile img')
const select = document.querySelector('.profile select')

username.addEventListener('input', e => {
    e.target.setCustomValidity('')
    
    if (e.target.value.trim().length > 25) return e.target.setCustomValidity('Your username must be 25 characters or less.\nPreceding or trailing whitespace will be trimmed.')
})

select.addEventListener('change', e => img.classList.value = e.target.value)

switch (mobile) {
    case true:
        document.querySelector('#device option[value="mobile"]').selected = true
    default:
        document.querySelector('#device option[value="desktop"]').selected = true
}