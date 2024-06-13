const searchLogger = new Logger('List', 'Search')

// keep in mind this does not make `levels` a static variable, all values are mutable and change
// with the page regardless of what is done. Code may look stupid on purpose
const levels = document.querySelectorAll('.level')
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
    const status = !term ? 'refresh' : `term "${term}"`

    if (elapsed > 30) searchLogger.warn(`${status} took ${elapsed}ms`)
    else searchLogger.log(`${status} took ${elapsed}ms`)

    // stats

    searchInfo.classList.toggle('visible', term)
    if (!term) return

    const results = document.querySelectorAll('.container > div:not(.search):not(.empty):not(.hidden)')

    searchInfo.innerHTML = `<b>${results.length}</b> result${results.length == 1 ? '' : 's'} in <b>${Math.round(elapsed) / 100}</b> seconds`
})

const imgLogger = new Logger('List', 'ImageLoader')

document.querySelectorAll('.level .img img').forEach(img => img.onload = handleImageLoad)

async function handleImageLoad(e) {
    if (e.target.naturalWidth != 120) return
    const videoid = e.target.src.match(/\/vi\/([^\/]+)/)[1]
    const res = e.target.src.match(/\/(\w+)default\.jpg$/)[1].toLowerCase()
    const split = e.target.src.split(/\w+(default\.jpg)$/)
    split.pop()

    switch (res) {
        case 'maxres':
            imgLogger.warn('Video ID', videoid, 'failed to load MAXRES, falling back to SD')
            e.target.src = split.join('sd')
            break
        case 'sd':
            imgLogger.warn('Video ID', videoid, 'failed to load SD, falling back to HQ')
            e.target.src = split.join('hq')
            break
    }
}