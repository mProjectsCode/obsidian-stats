import { type Parser } from '@lemons_dev/parsinom/lib/Parser';
import { P } from '@lemons_dev/parsinom/lib/ParsiNOM';
import { P_UTILS } from '@lemons_dev/parsinom/lib/ParserUtils';

const versionParser: Parser<Version> = P.sequenceMap(
	(_, a, b, c, d) => {
		const major = parseInt(a);
		const minor = parseInt(b);
		const patch = c ? parseInt(c) : 0;

		return new Version(major, minor, patch, d);
	},
	P.string('v').optional(),
	P_UTILS.digits(),
	P.string('.').then(P_UTILS.digits()),
	P.string('.').then(P_UTILS.digits()).optional(),
	P.string('-').then(P_UTILS.remaining()).optional(),
);

export class Version {
	readonly major: number;
	readonly minor: number;
	readonly patch: number;
	readonly prerelease?: string | undefined;

	constructor(major: number, minor: number, patch: number, prerelease?: string | undefined) {
		this.major = major;
		this.minor = minor;
		this.patch = patch;
		this.prerelease = prerelease;
	}

	public static fromString(version: string): Version {
		return versionParser.parse(version);
	}

	public static valid(version: string): boolean {
		return versionParser.tryParse(version).success;
	}

	public static pretty(version: string): string {
		const v = Version.fromString(version);
		return v.toString();
	}

	public static alphabetic(version: string): string {
		const v = Version.fromString(version);
		return `${v.major.toString().padStart(2, '0')}.${v.minor.toString().padStart(2, '0')}.${v.patch.toString().padStart(2, '0')}${v.prerelease === undefined ? '' : `-${v.prerelease}`}`;
	}

	public static lessThan(a: Version, b: Version): boolean {
		if (a.major < b.major) {
			return true;
		} else if (a.major > b.major) {
			return false;
		}

		if (a.minor < b.minor) {
			return true;
		} else if (a.minor > b.minor) {
			return false;
		}

		if (a.patch < b.patch) {
			return true;
		} else if (a.patch > b.patch) {
			return false;
		}

		if (a.prerelease === undefined) {
			return false;
		} else if (b.prerelease === undefined) {
			return true;
		}

		return a.prerelease < b.prerelease;
	}

	public static greaterThan(a: Version, b: Version): boolean {
		return Version.lessThan(b, a);
	}

	public static equals(a: Version, b: Version): boolean {
		return a.major === b.major && a.minor === b.minor && a.patch === b.patch && a.prerelease === b.prerelease;
	}

	toString(): string {
		return `${this.major}.${this.minor}.${this.patch}${this.prerelease === undefined ? '' : `-${this.prerelease}`}`;
	}

	getMajor(): Version {
		return new Version(this.major, 0, 0);
	}

	getMinor(): Version {
		return new Version(this.major, this.minor, 0);
	}

	getPatch(): Version {
		return new Version(this.major, this.minor, this.patch);
	}
}