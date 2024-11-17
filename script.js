// Get references to DOM elements
const currentMonthElem = document.getElementById('current-month');
const calendarDaysElem = document.getElementById('calendar-days');
const prevMonthButton = document.getElementById('prev-month');
const nextMonthButton = document.getElementById('next-month');

// Date variables
let currentDate = new Date();

function renderCalendar() {
    // Set the current month and year
    const month = currentDate.getMonth();
    const year = currentDate.getFullYear();

    const monthNames = [
        'January', 'February', 'March', 'April', 'May',
        'June', 'July', 'August', 'September', 'October',
        'November', 'December'
    ];
    currentMonthElem.textContent = `${monthNames[month]} ${year}`;

    // Clear previous calendar days
    calendarDaysElem.innerHTML = '';

    // Get the first day of the month and the last date of the month
    const firstDay = new Date(year, month, 1).getDay(); // Day of the week for the 1st
    const lastDate = new Date(year, month + 1, 0).getDate(); // Last day of the month

    // Total cells to render (empty + days + possible trailing empty cells)
    const totalCells = Math.ceil((firstDay + lastDate) / 7) * 7;

    // Fill in the grid with empty spaces and days
    for (let i = 0; i < totalCells; i++) {
        const dayDiv = document.createElement('div');

        // Fill in actual days of the month
        if (i >= firstDay && i < firstDay + lastDate) {
            const day = i - firstDay + 1;
            dayDiv.textContent = day;
            dayDiv.classList.add('day');

            // Highlight today's date
            const today = new Date();
            if (
                day === today.getDate() &&
                month === today.getMonth() &&
                year === today.getFullYear()
            ) {
                dayDiv.classList.add('today');
            }
        } else {
            // Empty cells for padding
            dayDiv.classList.add('empty');
        }

        calendarDaysElem.appendChild(dayDiv);
    }
}

// Event listeners for navigation buttons
prevMonthButton.addEventListener('click', () => {
    currentDate.setMonth(currentDate.getMonth() - 1);
    renderCalendar();
});

nextMonthButton.addEventListener('click', () => {
    currentDate.setMonth(currentDate.getMonth() + 1);
    renderCalendar();
});

// Initial render
renderCalendar();