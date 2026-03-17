export interface TestKanji {
  character: string;
  meanings: string[];
  onyomi: string[];
  kunyomi: string[];
  level: 'N5' | 'N4' | 'N3' | 'N2' | 'N1';
}

export interface TestVocabulary {
  word: string;
  readings: string[];
  meanings: string[];
  level: 'N5' | 'N4' | 'N3' | 'N2' | 'N1';
}

export interface TestGrammarRule {
  name: string;
  structure: string;
  meaning: string;
  level: 'N5' | 'N4' | 'N3' | 'N2' | 'N1';
}

export const TEST_KANJI: TestKanji[] = [
  { character: '日', meanings: ['день', 'солнце'], onyomi: ['ニチ', 'ジツ'], kunyomi: ['ひ', '-び', '-か'], level: 'N5' },
  { character: '月', meanings: ['месяц', 'луна'], onyomi: ['ゲツ', 'ガツ'], kunyomi: ['つき'], level: 'N5' },
  { character: '火', meanings: ['огонь'], onyomi: ['カ'], kunyomi: ['ひ', '-び', 'ほ-'], level: 'N5' },
  { character: '水', meanings: ['вода'], onyomi: ['スイ'], kunyomi: ['みず'], level: 'N5' },
  { character: '木', meanings: ['дерево', 'деревянный'], onyomi: ['ボク', 'モク'], kunyomi: ['き', 'こ-'], level: 'N5' },
  { character: '金', meanings: ['золото', 'металл', 'деньги'], onyomi: ['キン', 'コン'], kunyomi: ['かね', 'かな-', 'がね-'], level: 'N5' },
  { character: '土', meanings: ['земля', 'почва'], onyomi: ['ド', 'ト'], kunyomi: ['つち'], level: 'N5' },
  { character: '人', meanings: ['человек'], onyomi: ['ジン', 'ニン'], kunyomi: ['ひと', '-り', '-と'], level: 'N5' },
  { character: '一', meanings: ['один'], onyomi: ['イチ', 'イツ'], kunyomi: ['ひと-', 'ひと.つ'], level: 'N5' },
  { character: '二', meanings: ['два'], onyomi: ['ニ'], kunyomi: ['ふた', 'ふた.つ'], level: 'N5' },
];

export const TEST_VOCABULARY: TestVocabulary[] = [
  { word: 'こんにちは', readings: ['こんにちは'], meanings: ['здравствуйте', 'добрый день'], level: 'N5' },
  { word: 'ありがとう', readings: ['ありがとう'], meanings: ['спасибо'], level: 'N5' },
  { word: 'すみません', readings: ['すみません'], meanings: ['извините', 'простите'], level: 'N5' },
  { word: 'はい', readings: ['はい'], meanings: ['да'], level: 'N5' },
  { word: 'いいえ', readings: ['いいえ'], meanings: ['нет'], level: 'N5' },
  { word: 'おはよう', readings: ['おはよう'], meanings: ['доброе утро'], level: 'N5' },
  { word: 'こんばんは', readings: ['こんばんは'], meanings: ['добрый вечер'], level: 'N5' },
  { word: 'さようなら', readings: ['さようなら'], meanings: ['до свидания'], level: 'N5' },
  { word: '日本', readings: ['にほん', 'にっぽん'], meanings: ['Япония'], level: 'N5' },
  { word: '日本人', readings: ['にほんじん'], meanings: ['японец'], level: 'N5' },
];

export const TEST_GRAMMAR_RULES: TestGrammarRule[] = [
  { name: '〜は〜です', structure: 'A は B です', meaning: 'A есть B (основная конструкция предложения)', level: 'N5' },
  { name: '〜は〜ではありません', structure: 'A は B ではありません', meaning: 'A не есть B (отрицание)', level: 'N5' },
  { name: '〜は〜でした', structure: 'A は B でした', meaning: 'A было B (прошедшее время)', level: 'N5' },
  { name: '〜を〜ます', structure: 'A を B ます', meaning: 'Делать B с объектом A', level: 'N5' },
  { name: '〜に行きます', structure: 'A に 行きます', meaning: 'Идти в A', level: 'N5' },
];

export const TEST_USER = {
  email: 'e2e@sample.com',
  password: '12345678',
};
