// this code is copied and modified from list.js

const boardLogger = new Logger('Leaderboard')

// keep in mind this does not make `records` a static variable, all values are mutable and change
// with the page regardless of what is done. Code may look stupid on purpose
const records = document.querySelectorAll('.players .table a')
let lastValueLength = 0

// i am NOT typing this out every time
String.prototype.highlight = function(term) {
    return this.replace(new RegExp(`(${term})`, 'ig'), '<span class="highlight">$1</span>')
}

document.querySelector('.search textarea').addEventListener('input', event => {
    // search algorithm

    const start = performance.now()

    const term = event.target.value.replace(/[/\-\\^$*+?.()|[\]{}]/g, '\\$&') // escape special regex chars [https://stackoverflow.com/a/3561711]
    const textAdded = (term.length - lastValueLength) > 0
    lastValueLength = term.length

    let empty = true

    records.forEach(record => {
        if (textAdded && record.classList.contains('hidden')) return // skip searching hidden results if text is added

        const holder = record.children[1].innerText.replace(/\n/g, '')

        const match = holder.match(new RegExp(term, 'i')) != null

        record.classList.toggle('hidden', !match)

        if (match) empty = false

        // much faster way to reset everything than doing it before checking
        // because we loop through everything anyway
        if (term == '') {
            record.children[1].innerHTML = holder

            return
        }

        record.children[1].innerHTML = holder.highlight(term)
    })

    document.querySelector('.players').classList.toggle('hidden', empty)
    document.querySelector('.empty').classList.toggle('hidden', !empty)

    const elapsed = performance.now() - start
    const status = term == '' ? 'refresh' : `term "${term}"`

    if (elapsed > 30) boardLogger.warn(`${status} took ${elapsed}ms`)
    else boardLogger.log(`${status} took ${elapsed}ms`)

    // stats

    searchInfo.classList.toggle('visible', term)
    if (!term) return

    const results = document.querySelectorAll('.players .table > a:not(.hidden)')

    searchInfo.innerHTML = `<b>${results.length}</b> result${results.length == 1 ? '' : 's'} in <b>${Math.round(elapsed) / 100}</b> seconds`
})