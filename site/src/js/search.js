const searchBar = document.querySelector('.search textarea')

;(async function() {
    if (!searchBar) return

    function handleInputChange() {
        const value = !searchBar.value.trim() ? searchBar.placeholder : searchBar.value

        const test = document.createElement('span')
        test.className = 'test'
        test.innerText = value

        searchBar.parentElement.appendChild(test)

        const height = test.getBoundingClientRect().height
        test.remove()
        
        searchBar.style.height = height + 'px'
    }

    searchBar.addEventListener('input', handleInputChange)
    window.addEventListener('resize', handleInputChange)

    handleInputChange()
})()