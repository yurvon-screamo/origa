import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class OnboardingPage extends BasePage {
	// Page structure
	readonly onboardingPage: Locator;
	readonly onboardingCard: Locator;
	readonly onboardingContent: Locator;
	readonly onboardingStepper: Locator;

	// Loading
	readonly onboardingLoading: Locator;
	readonly onboardingSpinner: Locator;

	// Navigation buttons
	readonly prevButton: Locator;
	readonly nextButton: Locator;
	readonly importButton: Locator;

	// Steps
	readonly introStep: Locator;
	readonly jlptStep: Locator;
	readonly appsStep: Locator;
	readonly progressStep: Locator;
	readonly summaryStep: Locator;

	// Intro step
	readonly introTitle: Locator;
	readonly introSubtitle: Locator;

	// JLPT step
	readonly jlptTitle: Locator;
	readonly jlptSubtitle: Locator;

	// Apps step
	readonly appsTitle: Locator;
	readonly appsSubtitle: Locator;

	// Summary step
	readonly summaryTitle: Locator;
	readonly summarySubtitle: Locator;

	constructor(page: Page) {
		super(page);

		// Page structure
		this.onboardingPage = page.getByTestId("onboarding-page");
		this.onboardingCard = page.getByTestId("onboarding-card");
		this.onboardingContent = page.getByTestId("onboarding-content");
		this.onboardingStepper = page.getByTestId("onboarding-stepper");

		// Loading
		this.onboardingLoading = page.getByTestId("onboarding-loading");
		this.onboardingSpinner = page.getByTestId("onboarding-spinner");

		// Navigation buttons
		this.prevButton = page.getByTestId("onboarding-prev");
		this.nextButton = page.getByTestId("onboarding-next");
		this.importButton = page.getByTestId("onboarding-import");

		// Steps
		this.introStep = page.getByTestId("intro-step");
		this.jlptStep = page.getByTestId("jlpt-step");
		this.appsStep = page.getByTestId("apps-step");
		this.progressStep = page.getByTestId("progress-step");
		this.summaryStep = page.getByTestId("summary-step");

		// Intro step
		this.introTitle = page.getByTestId("intro-title");
		this.introSubtitle = page.getByTestId("intro-subtitle");

		// JLPT step
		this.jlptTitle = page.getByTestId("jlpt-title");
		this.jlptSubtitle = page.getByTestId("jlpt-subtitle");

		// Apps step
		this.appsTitle = page.getByTestId("apps-title");
		this.appsSubtitle = page.getByTestId("apps-subtitle");

		// Summary step
		this.summaryTitle = page.getByTestId("summary-title");
		this.summarySubtitle = page.getByTestId("summary-subtitle");
	}

	async goto(): Promise<void> {
		await this.navigate("/onboarding");
	}

	async expectOnboardingVisible(): Promise<void> {
		await expect(this.onboardingPage).toBeVisible();
		await expect(this.onboardingCard).toBeVisible();
	}

	async selectJlptLevel(level: string): Promise<void> {
		// level can be "N5", "N4", "N3", "N2", "N1", or "unknown"
		const levelOption = this.page.locator(`text="${level}"`);
		await levelOption.click();
	}

	async toggleApp(appId: string): Promise<void> {
		const appCheckbox = this.page.getByTestId(`apps-checkbox-${appId}`);
		await appCheckbox.click();
	}

	async toggleSet(setId: string): Promise<void> {
		const setCheckbox = this.page.getByTestId(`summary-checkbox-${setId}`);
		await setCheckbox.click();
	}

	async goToNextStep(): Promise<void> {
		await this.nextButton.click();
	}

	async goToPrevStep(): Promise<void> {
		await this.prevButton.click();
	}

	async startImport(): Promise<void> {
		await this.importButton.click();
	}

	async expectStepVisible(stepTestId: string): Promise<void> {
		await expect(this.page.getByTestId(stepTestId)).toBeVisible();
	}
}
