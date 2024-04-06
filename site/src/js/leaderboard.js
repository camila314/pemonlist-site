// all of this code is modified from list.js

const records = document.querySelectorAll('.records .table a')
let lastValueLength = 0

String.prototype.highlight = function(term) {
    return this.replace(new RegExp(`(\s)?(${term})`, 'ig'), '<span class="highlight">$1$2</span>')
}

document.querySelector('.search textarea').addEventListener('input', event => {
    const term = event.target.value.replace(/[/\-\\^$*+?.()|[\]{}]/g, '\\$&')
    const textAdded = (term.length - lastValueLength) > 0
    lastValueLength = term.length

    let empty = true

    records.forEach(record => {
        if (textAdded && record.classList.contains('hidden')) return

        const holder = record.children[1].innerText.replace(/\n/g, '')
        
        const match = holder.match(new RegExp(term, 'i')) != null

        record.classList.toggle('hidden', !match)

        if (match) empty = false

        record.children[1].innerHTML = holder.highlight(term)
    })

    document.querySelector('.empty').classList.toggle('hidden', !empty)
    document.querySelector('.records').classList.toggle('hidden', empty)
})
