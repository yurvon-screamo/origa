import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class HomePage extends BasePage {
  // Page structure
  readonly homePage: Locator;
  readonly homeContent: Locator;

  // Welcome card (Hero block)
  readonly welcomeCard: Locator;

  // Sidebar navigation
  readonly sidebar: Locator;
  readonly sidebarAvatar: Locator;
  readonly sidebarHome: Locator;
  readonly sidebarWords: Locator;
  readonly sidebarGrammar: Locator;
  readonly sidebarKanji: Locator;

  // JLPT Progress
  readonly jlptProgress: Locator;
  readonly jlptStamp: Locator;

  // Lesson
  readonly lessonButton: Locator;

  // Accuracy chart (fl2O037UR7)
  readonly accuracyChart: Locator;
  readonly accuracyChartPlot: Locator;

  // Recent Study
  readonly recentStudy: Locator;

  // Bottom tab bar navigation (mobile)
  readonly bottomTabBar: Locator;
  readonly bottomTabHome: Locator;
  readonly bottomTabWords: Locator;
  readonly bottomTabGrammar: Locator;
  readonly bottomTabKanji: Locator;
  readonly bottomTabProfile: Locator;

  // Loading
  readonly homeLoading: Locator;
  readonly homeSpinner: Locator;

  constructor(page: Page) {
    super(page);

    // Page structure
    this.homePage = page.getByTestId("home-page");
    this.homeContent = page.getByTestId("home-content");

    // Welcome card
    this.welcomeCard = page.getByTestId("home-welcome");

    // Sidebar navigation
    this.sidebar = page.getByTestId("sidebar");
    this.sidebarAvatar = page.getByTestId("sidebar-avatar");
    this.sidebarHome = page.getByTestId("sidebar-tab-home");
    this.sidebarWords = page.getByTestId("sidebar-tab-words");
    this.sidebarGrammar = page.getByTestId("sidebar-tab-grammar");
    this.sidebarKanji = page.getByTestId("sidebar-tab-kanji");

    // JLPT Progress
    this.jlptProgress = page.getByTestId("home-jlpt-progress");
    this.jlptStamp = page.getByTestId("home-jlpt-progress-stamp");

    // Lesson
    this.lessonButton = page.getByTestId("home-welcome-lesson");

    // Accuracy chart (fl2O037UR7)
    this.accuracyChart = page.getByTestId("home-accuracy-chart");
    this.accuracyChartPlot = page.getByTestId("home-accuracy-chart-chart");

    // Recent Study
    this.recentStudy = page.getByTestId("home-recent-study");

    // Bottom tab bar navigation (mobile)
    this.bottomTabBar = page.locator(".bottom-tab-bar");
    this.bottomTabHome = page.getByTestId("bottom-tab-tab-home");
    this.bottomTabWords = page.getByTestId("bottom-tab-tab-words");
    this.bottomTabGrammar = page.getByTestId("bottom-tab-tab-grammar");
    this.bottomTabKanji = page.getByTestId("bottom-tab-tab-kanji");
    this.bottomTabProfile = page.getByTestId("bottom-tab-tab-profile");

    // Loading
    this.homeLoading = page.getByTestId("home-loading");
    this.homeSpinner = page.getByTestId("home-spinner");
  }

  async goto(): Promise<void> {
    await this.navigate("/home");
  }

  async expectHomeVisible(): Promise<void> {
    await expect(this.homePage).toBeVisible();
    await expect(this.homeContent).toBeVisible();
  }

  async navigateToWords(): Promise<void> {
    await this.page.goto("/words");
  }

  async navigateToGrammar(): Promise<void> {
    await this.page.goto("/grammar");
  }

  async navigateToKanji(): Promise<void> {
    await this.page.goto("/kanji");
  }

  async startLesson(): Promise<void> {
    await this.lessonButton.click();
  }

  async navigateViaBottomTab(
    tabName: "home" | "words" | "grammar" | "kanji" | "profile",
  ): Promise<void> {
    const tabMap: Record<typeof tabName, Locator> = {
      home: this.bottomTabHome,
      words: this.bottomTabWords,
      grammar: this.bottomTabGrammar,
      kanji: this.bottomTabKanji,
      profile: this.bottomTabProfile,
    };
    await tabMap[tabName].click();
  }
}
