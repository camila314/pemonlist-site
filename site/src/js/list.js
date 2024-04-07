const listLogger = new Logger('List')

// keep in mind this does not make `levels` a static variable, all values are mutable and change
// with the page regardless of what is done. Code may look stupid on purpose
const levels = document.querySelectorAll('.level')
let lastValueLength = 0

// i am NOT typing this out every time
String.prototype.highlight = function(term) {
    return this.replace(new RegExp(`(${term})`, 'ig'), '<span class="highlight">$1</span>')
}

document.querySelector('.search textarea').addEventListener('input', event => {
    const start = performance.now()

    const term = event.target.value.replace(/[/\-\\^$*+?.()|[\]{}]/g, '\\$&') // escape special regex chars [https://stackoverflow.com/a/3561711]
    const textAdded = (term.length - lastValueLength) > 0
    lastValueLength = term.length

    let empty = true

    levels.forEach(level => {
        if (textAdded && level.classList.contains('hidden')) return // skip searching hidden results if text is added

        const placement = level.children[2].innerText.replace(/\n/g, '')
        const title = level.children[1].children[0].innerText.replace(/\n/g, '')
        const author = level.children[1].children[1].innerText.replace(/\n/g, '')
        const search = placement + title + author // combine search string for faster checking

        const match = search.match(new RegExp(term, 'i')) != null

        level.classList.toggle('hidden', !match)

        if (match) empty = false

        // much faster way to reset everything than doing it before checking
        // because we loop through everything anyway
        if (term == '') {
            level.children[2].innerHTML = placement
            level.children[1].children[0].innerHTML = title
            level.children[1].children[1].innerHTML = author

            return
        }

        level.children[2].innerHTML = placement.highlight(term)
        level.children[1].children[0].innerHTML = title.highlight(term)
        level.children[1].children[1].innerHTML = author.highlight(term)
    })

    document.querySelector('.empty').classList.toggle('hidden', !empty)

    const elapsed = performance.now() - start
    const status = term == '' ? 'refresh' : `term "${term}"`

    if (elapsed > 30) listLogger.warn(`${status} took ${elapsed}ms`)
    else listLogger.log(`${status} took ${elapsed}ms`)
})
