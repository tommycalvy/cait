(function () {
    // Scroll to bottom of the target when there is a htmx:sseMessage event

    htmx.defineExtension('scroll-bottom', {
        onEvent: function (name, evt) {
            if (name === "htmx:sseMessage") {
                var elt = evt.detail.elt;
                if (elt.getAttribute) {
                    var selector = elt.getAttribute("scroll-bottom");
                    if (selector) {
                        var target = document.getElementById(selector);
                        if (target) {
                            var rect = target.getBoundingClientRect();
                            var elemTop = rect.top;
                            var elemBottom = rect.bottom;
                            if (elemTop < window.innerHeight && elemBottom >= 0) {
                                target.scrollIntoView({ block: "end", behavior: htmx.config.scrollBehavior });
                            }
                        }
                    }
                }
            }
        }
    });
})();