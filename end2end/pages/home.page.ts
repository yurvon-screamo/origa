import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class HomePage extends BasePage {
    // Page structure
    readonly homePage: Locator;
    readonly homeContent: Locator;
    readonly homeHeader: Locator;

    // Header navigation
    readonly homeAvatar: Locator;
    readonly homeWords: Locator;
    readonly homeGrammar: Locator;
    readonly homeKanji: Locator;

    // JLPT Progress
    readonly jlptProgress: Locator;
    readonly jlptStamp: Locator;

    // Stats
    readonly statsGrid: Locator;
    readonly lessonButtons: Locator;
    readonly lessonButton: Locator;
    // Stat cards
    readonly statLearned: Locator;
    readonly statInProgress: Locator;
    readonly statNew: Locator;
    readonly statHighDifficulty: Locator;
    readonly statPositive: Locator;
    readonly statNegative: Locator;
    readonly statTotalRatings: Locator;

    // History modal
    readonly historyModal: Locator;
    readonly historyChart: Locator;

    // Navigation
    readonly navDrawer: Locator;
    readonly homeHamburger: Locator;
    readonly navDrawerLesson: Locator;
    readonly navDrawerProfile: Locator;
    readonly navDrawerWords: Locator;

    // Loading
    readonly homeLoading: Locator;
    readonly homeSpinner: Locator;

    // Stats toggle
    readonly toggleDetails: Locator;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.homePage = page.getByTestId("home-page");
        this.homeContent = page.getByTestId("home-content");
        this.homeHeader = page.getByTestId("home-header");

        // Header navigation
        this.homeAvatar = page.getByTestId("home-header-avatar");
        this.homeWords = page.getByTestId("home-header-words");
        this.homeGrammar = page.getByTestId("home-header-grammar");
        this.homeKanji = page.getByTestId("home-header-kanji");

        // JLPT Progress
        this.jlptProgress = page.getByTestId("home-jlpt-progress");
        this.jlptStamp = page.getByTestId("home-jlpt-progress-stamp");

        // Stats
        this.statsGrid = page.getByTestId("home-stats-grid");
        this.lessonButtons = page.getByTestId("lesson-buttons");
        this.lessonButton = page.getByTestId("lesson-buttons-lesson");

        // Stat cards
        this.statLearned = page.getByTestId("stat-learned");
        this.statInProgress = page.getByTestId("stat-in-progress");
        this.statNew = page.getByTestId("stat-new");
        this.statHighDifficulty = page.getByTestId("stat-high-difficulty");
        this.statPositive = page.getByTestId("stat-positive");
        this.statNegative = page.getByTestId("stat-negative");
        this.statTotalRatings = page.getByTestId("stat-total-ratings");

        // History modal
        this.historyModal = page.getByTestId("home-history-modal");
        this.historyChart = page.getByTestId("history-chart");

        // Navigation
        this.navDrawer = page.getByTestId("nav-drawer");
        this.homeHamburger = page.getByTestId("home-hamburger");
        this.navDrawerLesson = page.getByTestId("nav-drawer-lesson");
        this.navDrawerProfile = page.getByTestId("nav-drawer-profile");
        this.navDrawerWords = page.getByTestId("nav-drawer-words");

        // Loading
        this.homeLoading = page.getByTestId("home-loading");
        this.homeSpinner = page.getByTestId("home-spinner");

        // Stats toggle
        this.toggleDetails = page.getByTestId("toggle-details");
    }

    async goto(): Promise<void> {
        await this.navigate("/home");
    }

    async expectHomeVisible(): Promise<void> {
        await expect(this.homePage).toBeVisible();
        await expect(this.homeContent).toBeVisible();
        await expect(this.homeHeader).toBeVisible();
    }

    async navigateToWords(): Promise<void> {
        await this.homeWords.click();
    }

    async navigateToGrammar(): Promise<void> {
        await this.homeGrammar.click();
    }

    async navigateToKanji(): Promise<void> {
        await this.homeKanji.click();
    }

    async startLesson(): Promise<void> {
        await this.lessonButton.click();
    }

    async openHistoryForStat(statTestId: string): Promise<void> {
        const statCard = this.page.getByTestId(statTestId);
        await statCard.click();
    }

    async toggleDetailedStats(): Promise<void> {
        await this.toggleDetails.click();
    }

    async openNavDrawer(): Promise<void> {
        await this.homeHamburger.click();
    }

    async navigateToProfileFromDrawer(): Promise<void> {
        await this.openNavDrawer();
        await this.navDrawerProfile.click();
    }

    async navigateToLessonFromDrawer(): Promise<void> {
        await this.openNavDrawer();
        await this.navDrawerLesson.click();
    }

    async navigateToWordsFromDrawer(): Promise<void> {
        await this.openNavDrawer();
        await this.navDrawerWords.click();
    }

    async closeNavDrawer(): Promise<void> {
        await this.page.keyboard.press("Escape");
    }
}
