htmx.isSwapping = false;

/********************\
| Menu Toggle Button |
\********************/

const hideMenuToggleButtonEventMap = new WeakMap();

/**
 * Implements the keyboard controls and attributes for a button that opens a popover menu
 */
function initializeMenuToggleButton(element) {
    element.removeAttribute('tabindex');
    element.classList.remove('button--has-focus-popover');
    element.classList.add('button--has-js-popover');
    const toggle = element.querySelector('[data-menu-toggle]');
    toggle.setAttribute('tabindex', '0');
    toggle.setAttribute('aria-expanded', 'false');
    toggle.removeAttribute('aria-hidden');
    const popover = element.querySelector('.popover');
    const menu = element.querySelector('[role="menu"]');
    menu.querySelectorAll('a').forEach((link) => {
        link.setAttribute('tabindex', '-1');
    });
    menu.querySelector('a').setAttribute('tabindex', '0');
    function togglePopover() {
        popover.classList.toggle('popover--show');
        if (popover.classList.contains('popover--show')) {
            toggle.setAttribute('aria-expanded', 'true');
            menu.querySelector('a[tabindex="0"]')?.focus();
        } else {
            toggle.setAttribute('aria-expanded', 'false');
            toggle.focus();
        }
    }
    element.addEventListener('click', (event) => {
        if (event.target === element || toggle.contains(event.target)) {
            togglePopover();
        }
    })
    toggle.addEventListener('keydown', (event) => {
        if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            togglePopover();
        }
    });
    menu.addEventListener('keydown', (event) => {
        let targetLink;
        if (event.key === 'ArrowUp' || event.key === 'ArrowLeft') {
            event.preventDefault();
            targetLink = event.target.closest('li')?.previousElementSibling?.querySelector('a');
        } else if (event.key === 'ArrowDown' || event.key === 'ArrowRight') {
            event.preventDefault();
            targetLink = event.target.closest('li')?.nextElementSibling?.querySelector('a');
        } else if (event.key === 'Home') {
            event.preventDefault();
            targetLink = menu.querySelector('a');
        } else if (event.key === 'End') {
            event.preventDefault();
            targetLink = menu.querySelector('li:last-child a');
        } else if (event.key === 'Escape') {
            event.preventDefault();
            togglePopover();
            toggle.focus();
        } else if (event.key === 'Enter' || event.key === ' ') {
            setTimeout(() => {
                togglePopover();
                toggle.focus();
            }, 0);
        }
        if (!targetLink) return;
        menu.querySelectorAll('a').forEach((link) => {
            link.setAttribute('tabindex', '-1');
        });
        targetLink.setAttribute('tabindex', '0');
        targetLink.focus();
    });
    menu.addEventListener('click', (event) => {
        if (event.target?.tagName === 'A') {
            togglePopover();
            toggle.focus();
        }
    });
    function hideToggleOnClickAway(event) {
        setTimeout(() => {
            const target = event.target || document.activeElement;
            if (!target || !element.contains(target)) {
                if (popover.classList.contains('popover--show')) {
                    toggle.click();
                }
            }
        }, 0);
    }
    hideMenuToggleButtonEventMap.set(element, hideToggleOnClickAway);
    document.addEventListener('mousedown', hideToggleOnClickAway, true);
}

/**
 * Removes the global event(s).
 */
function teardownMenuToggleButton(element) {
    const callback = hideMenuToggleButtonEventMap.get(element);
    if (!callback) return;
    document.removeEventListener('mousedown', callback, true);
}

/*****************\
| Site Navigation |
\*****************/

/**
 * Style the navigation link that corresponds to the currently active
 * section of the site. (Home, Bio, etc.)
 */
function setActiveSiteNavigationLink(detail) {
    // If the UI shows the user as authenticated but the response says the user isn't, refresh the page.
    if (detail?.detail?.xhr?.getResponseHeader('X-Authenticated') == 'false' && document.getElementById('site-sign-out-form')) {
        window.location.reload();
        return;
    }
    // If the hx-refresh query param exists, refresh the page.
    let params = new URLSearchParams(window.location.search);
    if (params.has('hx-refresh')) {
        params.delete('hx-refresh');
        window.location.search = params.toString();
    }

    if (detail && ![document.body, document.getElementById('main-article')].includes(detail?.target)) return;
    const activeClass = 'header__navbar__item--active';
    htmx.removeClass(htmx.find(`#site-header-navbar .${activeClass}`), activeClass);
    let pathName = window.location.pathname.replace('.php', '').split('/').slice(1, 2)[0];
    if (pathName === '') pathName = 'home';
    const activeNavItem = htmx.find(`#site-header-nav-item-${pathName}`);
    if (activeNavItem) {
        htmx.addClass(activeNavItem, activeClass);
        // If the sidebar is gone, refresh the page
        const sidebar = document.getElementById('main-aside');
        if (!sidebar) window.location.reload();
    }
    if (detail) {
        if (activeNavItem) {
            document.title = `${activeNavItem.querySelector('.header__navbar__title')?.textContent} - SupercarBand.com`;
        } else {
            document.title = `${document.querySelector('h1')?.textContent ?? ''} - SupercarBand.com`;
        }
    }
    document.body.classList.remove('viewer-open');
    document.dispatchEvent(new Event('supercar:navigated'));
}
document.addEventListener('htmx:historyRestore', () => setActiveSiteNavigationLink());
document.addEventListener('htmx:afterSwap', setActiveSiteNavigationLink);
setActiveSiteNavigationLink();

/******\
| Tabs |
\******/

/**
 * Implements keyboard controls for tabs, that only Javascript can enable.
 */
function initializeTabs(element) {
    element.addEventListener('keydown', (event) => {
        if (event.key === 'ArrowRight' || event.key === 'ArrowDown') selectNextTab();
        else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') selectPreviousTab();
    });
    function selectTab(newElement, oldElement) {
        oldElement.classList.remove('tabs__tab--current');
        oldElement.setAttribute('tabindex', '-1');
        newElement.classList.remove('tabs__tab--current');
        newElement.removeAttribute('tabindex');
        newElement.focus();
        newElement.click();
    }
    function selectNextTab() {
        const selectedElement = element.querySelector('.tabs__tab--current');
        if (selectedElement.nextElementSibling) {
            selectTab(selectedElement.nextElementSibling, selectedElement);
        }
    }
    function selectPreviousTab() {
        const selectedElement = element.querySelector('.tabs__tab--current');
        if (selectedElement.previousElementSibling) {
            selectTab(selectedElement.previousElementSibling, selectedElement);
        }
    }
    element.querySelectorAll('.tabs__tab').forEach((element) => {
        element.setAttribute('tabindex', '-1');
    });
    element.querySelector('.tabs__tab--current')?.removeAttribute('tabindex');
}

/************\
| Timestamps |
\************/

/**
 * Add a timestamp inside of <time> that matches the current user's time zone and locale.
 */
function initializeTimestamp(element) {
    const dateTime = element.getAttribute('datetime');
    if (!dateTime) return;
    if (dateTime.includes('T')) {
        element.textContent = new Date(dateTime).toLocaleString(undefined, {
            year: "numeric", month: "long", day: "numeric",
            hour: 'numeric', minute: '2-digit'
        });
    } else {
        const now = new Date();
        const isoString = now.toISOString();
        const offsetMinutes = now.getTimezoneOffset();
        const sign = offsetMinutes > 0 ? '-' : '+';
        const offsetHours = String(Math.floor(Math.abs(offsetMinutes) / 60)).padStart(2, '0');
        const offsetMins = String(Math.abs(offsetMinutes) % 60).padStart(2, '0');
        const isoWithTimezone = `${isoString.replace("Z", "")}${sign}${offsetHours}:${offsetMins}`;
        const time = isoWithTimezone.split('T')[1];
        element.textContent = new Date(dateTime + 'T' + time).toLocaleString(undefined, {
            year: "numeric", month: "long", day: "numeric",
        });
    }
}

/**************************\
| Component Initialization |
\**************************/

function initializeComponents(eventName, detail) {
    if (eventName === 'afterSettle') {
        htmx.isSwapping = false;
    }
    
    const target = detail?.target ?? document.body;
    target.querySelectorAll('[data-is]').forEach((element) => {
        const is = element.getAttribute('data-is');
        switch (is) {
            case 'menu-toggle-button': initializeMenuToggleButton(element); break;
            case 'tabs': initializeTabs(element); break;
            case 'timestamp': initializeTimestamp(element); break;
        }
    });
}
function teardownComponents(eventName, detail) {
    if (eventName === 'beforeSwap') {
        htmx.isSwapping = true;
    }

    const target = detail?.target ?? document.body;
    target.querySelectorAll('[data-is]').forEach((element) => {
        const is = element.getAttribute('data-is');
        switch (is) {
            case 'menu-toggle-button': teardownMenuToggleButton(element); break;
        }
    });
}
document.addEventListener('htmx:beforeHistorySave', () => teardownComponents('beforeHistorysave'));
document.addEventListener('htmx:historyRestore', () => initializeComponents('historyRestore'));
document.addEventListener('htmx:beforeSwap', (event) => teardownComponents('beforeSwap', event));
document.addEventListener('htmx:afterSettle', (event) => initializeComponents('afterSettle', event));
document.addEventListener('htmx:oobBeforeSwap', (event) => teardownComponents('oobBeforeSwap', event));
document.addEventListener('htmx:oobAfterSwap', (event) => initializeComponents('oobAfterSwap', event));
initializeComponents();
