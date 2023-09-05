(function () {

    // Set theme by changing the classes on the root element
    htmx.defineExtension('set-theme', {
        onEvent: function (name, evt) {
            if (name === "htmx:afterProcessNode") {
                var elt = evt.detail.elt;
                if (elt.getAttribute) {
                    var theme = elt.getAttribute("set-theme");
                    if (theme) {
                        document.documentElement.className = '';
                        document.documentElement.classList.add(theme);
                    }
                }
            }
        }
    });
})();