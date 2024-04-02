// keep in mind this does not make `levels` a static variable, all values are mutable and change
// with the page regardless of what is done. Code may look stupid on purpose
const levels = document.querySelectorAll('.container div:not(.search)')
let lastValueLength = 0

// i am NOT typing this out every time
String.prototype.highlight = function(term) {
    return this.replace(new RegExp(`(${term})`, 'ig'), '<span class="highlight">$1</span>')
}

console.info('---------- Highlighting Benchmark ----------')
// console.info('---------- No Highlighting Benchmark ----------')
// console.info('----------- (w/o empty term checks) -----------')
// console.info('------------- (empty term checks) -------------')

document.querySelector('.search textarea').addEventListener('input', event => {
    const now = performance.now()
    const term = event.target.value.replace(/[/\-\\^$*+?.()|[\]{}]/g, '\\$&') // escape special regex chars [https://stackoverflow.com/a/3561711]
    const textAdded = (term.length - lastValueLength) >= 0
    lastValueLength = term.length

    // resetting everything when the search term is empty has no effect when highlighting,
    // as having to reset the highlights as well is the same if not slower

    // if (term == '') {
    //     levels.forEach(level => level.classList.remove('hidden'))
    //     return console.log('refresh', 'took', performance.now() - now + 'ms')
    // }

    levels.forEach(level => {
        if (textAdded && level.classList.contains('hidden')) return // skip searching hidden results if text is added

        const placement = level.children[2].innerText
        const title = level.children[1].children[0].innerText
        const author = level.children[1].children[1].innerText
        const search = placement + title + author // combine search string for faster checking

        const match = search.match(new RegExp(term, 'i')) != null

        level.classList.toggle('hidden', !match)

        level.children[2].innerHTML = placement.highlight(term)
        level.children[1].children[0].innerHTML = title.highlight(term)
        level.children[1].children[1].innerHTML = author.highlight(term)
    })

    console.log(term == '' ? 'refresh' : term, 'took', performance.now() - now + 'ms')
})
