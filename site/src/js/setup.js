const img = document.querySelector('.profile img')
const select = document.querySelector('.profile select')

select.addEventListener('change', e => img.classList.value = e.target.value)