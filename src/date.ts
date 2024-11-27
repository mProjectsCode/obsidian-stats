function UTCDate(year: number, month: number, date: number): Date {
	return new Date(Date.UTC(year, month - 1, date));
}

export class CDate {
	year: number;
	month: number;
	date: number;

	constructor(year: number, month: number, date: number) {
		this.year = year;
		this.month = month;
		this.date = date;
	}

	static fromString(str: string): CDate {
		const [year, month, date] = str.split('-').map(x => parseInt(x));
		return new CDate(year, month, date);
	}
	static fromMonthString(str: string): CDate {
		const [year, month, date] = str.split('-').map(x => parseInt(x));
		return new CDate(year, month, date);
	}

	static fromDate(date: Date): CDate {
		return new CDate(date.getFullYear(), date.getMonth() + 1, date.getDate());
	}

	static fromNow(): CDate {
		return CDate.fromDate(new Date());
	}

	static clone(date: CDate): CDate {
		return new CDate(date.year, date.month, date.date);
	}

	yearString(): string {
		return this.year.toString();
	}

	monthString(): string {
		return this.month.toString().padStart(2, '0');
	}

	dateString(): string {
		return this.date.toString().padStart(2, '0');
	}

	monthName(): string {
		const date = this.toDate();
		return date.toLocaleString('default', { month: 'long' });
	}

	monthNameFormat(): string {
		const date = this.toDate();
		return date.toLocaleString('default', { month: 'long', year: 'numeric' });
	}

	toString(): string {
		return `${this.yearString()}-${this.monthString()}-${this.dateString()}`;
	}

	toMonthString(): string {
		return `${this.yearString()}-${this.monthString()}`;
	}

	toDate(): Date {
		return new Date(this.toUTC());
	}

	toUTC(): number {
		return Date.UTC(this.year, this.month - 1, this.date);
	}

	setFromDate(date: Date): void {
		this.year = date.getFullYear();
		this.month = date.getMonth() + 1;
		this.date = date.getDate();
	}

	biggerThan(other: CDate): boolean {
		if (this.year > other.year) {
			return true;
		} else if (this.year < other.year) {
			return false;
		}

		if (this.month > other.month) {
			return true;
		} else if (this.month < other.month) {
			return false;
		}

		if (this.date > other.date) {
			return true;
		} else if (this.date < other.date) {
			return false;
		}

		return false;
	}

	smallerThan(other: CDate): boolean {
		return other.biggerThan(this);
	}

	equals(other: CDate): boolean {
		return this.year === other.year && this.month === other.month && this.date === other.date;
	}

	inRange(start: CDate, end: CDate): boolean {
		if (this.smallerThan(start)) {
			return false;
		}

		return this.smallerThan(end);
	}

	advanceByDays(days: number): void {
		const date = this.toDate();
		date.setDate(date.getDate() + days);
		this.setFromDate(date);
	}

	advanceDay(): void {
		this.advanceByDays(1);
	}

	advanceWeek(): void {
		this.advanceByDays(7);
	}

	advanceMonth(): void {
		const date = this.toDate();
		date.setMonth(date.getMonth() + 1);
		this.setFromDate(date);
	}

	advanceToWeekDay(weekDay: number): void {
		const date = this.toDate();
		const dayOffset = (weekDay - date.getDay() + 7) % 7;
		this.advanceByDays(dayOffset);
	}

	advanceToNextSunday(): void {
		this.advanceToWeekDay(0);
	}

	static dateDiffInDays(a: CDate, b: CDate): number {
		const utc1 = a.toUTC();
		const utc2 = b.toUTC();

		const _MS_PER_DAY = 1000 * 60 * 60 * 24;

		return Math.floor((utc2 - utc1) / _MS_PER_DAY);
	}

	static iterateDaily<T>(start: CDate, end: CDate, callback: (date: CDate) => T): T[] {
		const result: T[] = [];

		const startDate = CDate.clone(start);
		const endDate = CDate.clone(end);

		while (startDate.smallerThan(endDate)) {
			result.push(callback(CDate.clone(startDate)));
			startDate.advanceDay();
		}

		return result;
	}

	static iterateWeekly<T>(start: CDate, end: CDate, callback: (date: CDate) => T): T[] {
		const result: T[] = [];

		const startDate = CDate.clone(start);
		const endDate = CDate.clone(end);

		startDate.advanceToNextSunday();
		endDate.advanceToNextSunday();
		endDate.advanceDay();

		while (startDate.smallerThan(endDate)) {
			result.push(callback(CDate.clone(startDate)));
			startDate.advanceWeek();
		}

		return result;
	}

	static iterateMonthly<T>(start: CDate, end: CDate, callback: (date: CDate) => T): T[] {
		const result: T[] = [];

		const startDate = CDate.clone(start);
		const endDate = CDate.clone(end);

		startDate.date = 1;
		endDate.date = 1;

		while (startDate.smallerThan(endDate)) {
			result.push(callback(CDate.clone(startDate)));
			startDate.advanceMonth();
		}

		return result;
	}
}
