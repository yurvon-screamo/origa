"""
Enrich vocabulary chunk_11.json with missing Japanese words.

Reads the existing chunk file, adds curated real vocabulary words
(excluding fragments, onomatopoeia, and character names), and writes back.
"""

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CHUNK_PATH = REPO_ROOT / "origa_ui" / "public" / "dictionary" / "vocabulary" / "chunk_11.json"

NEW_WORDS: dict[str, dict[str, str]] = {
    # === Verbs (potential forms, colloquial forms) ===
    "言える": {
        "russian_translation": "можно сказать; в состоянии сказать\n\n> потенциальная форма глагола 言う (сказать).",
        "english_translation": "can say; able to say\n\n> potential form of 言う (to say).",
    },
    "思える": {
        "russian_translation": "можно подумать; представляется возможным; кажется\n\n> потенциальная форма глагола 思う (думать).",
        "english_translation": "can think; seems possible; appears to be\n\n> potential form of 思う (to think).",
    },
    "気づく": {
        "russian_translation": "заметить; осознать; обратить внимание\n\n> глагол, означающий внезапное осознание чего-либо.",
        "english_translation": "to notice; to realize; to become aware of\n\n> verb meaning sudden awareness or realization.",
    },
    "抑える": {
        "russian_translation": "подавлять; сдерживать; укрощать; контролировать\n\n> используется как в прямом (физически удерживать), так и в переносном смысле (сдерживать эмоции).",
        "english_translation": "to suppress; to restrain; to control; to hold back\n\n> used both literally (physically hold down) and figuratively (suppress emotions).",
    },
    "淹れる": {
        "russian_translation": "заваривать (чай); наливать (напиток)\n\n> специфический глагол для заваривания чая, не путать с 入れる.",
        "english_translation": "to brew (tea); to pour (a beverage)\n\n> specifically used for brewing tea; distinct from 入れる.",
    },
    "会える": {
        "russian_translation": "можно встретить; суметь увидеться\n\n> потенциальная форма глагола 会う (встречаться).",
        "english_translation": "can meet; able to see (someone)\n\n> potential form of 会う (to meet).",
    },
    "動ける": {
        "russian_translation": "может двигаться; в состоянии действовать\n\n> потенциальная форма глагола 動く (двигаться).",
        "english_translation": "can move; able to act\n\n> potential form of 動く (to move).",
    },
    "出せる": {
        "russian_translation": "может вывести; в состоянии выставить; способен произвести\n\n> потенциальная форма глагола 出す (вынимать, выводить).",
        "english_translation": "can take out; able to produce; can put out\n\n> potential form of 出す (to take out, to produce).",
    },
    "勝てる": {
        "russian_translation": "может победить; в состоянии выиграть\n\n> потенциальная форма глагола 勝つ (побеждать).",
        "english_translation": "can win; able to defeat\n\n> potential form of 勝つ (to win).",
    },
    "訊く": {
        "russian_translation": "спрашивать; осведомляться\n\n> кандзи 訊 используется для обозначения вопроса с целью получения информации.",
        "english_translation": "to ask; to inquire\n\n> kanji 訊 specifically denotes asking to obtain information.",
    },
    "愛す": {
        "russian_translation": "любить; испытывать глубокую привязанность\n\n> глагол суру-типа от 愛 (любовь). Более литературный, чем 愛する.",
        "english_translation": "to love; to cherish\n\n> suru-verb form of 愛 (love). More literary than 愛する.",
    },
    "熟す": {
        "russian_translation": "созревать; достигать зрелости; дозревать\n\n> используется для описания процесса созревания плодов, идей или отношений.",
        "english_translation": "to ripen; to mature; to come to fruition\n\n> used for fruit, ideas, or relationships reaching maturity.",
    },
    "聞ける": {
        "russian_translation": "может слышать; в состоянии слушать; можно спросить\n\n> потенциальная форма глагола 聞く (слышать, спрашивать).",
        "english_translation": "can hear; able to listen; can ask\n\n> potential form of 聞く (to hear/listen, to ask).",
    },
    "眠れる": {
        "russian_translation": "может уснуть; в состоянии спать\n\n> потенциальная форма глагола 眠る (спать).",
        "english_translation": "can sleep; able to fall asleep\n\n> potential form of 眠る (to sleep).",
    },
    "許せる": {
        "russian_translation": "может простить; в состоянии позволить\n\n> потенциальная форма глагола 許す (прощать, разрешать).",
        "english_translation": "can forgive; able to permit\n\n> potential form of 許す (to forgive, to permit).",
    },
    "付きあう": {
        "russian_translation": "общаться; встречаться (романтически); поддерживать отношения\n\n> составной глагол: 付く (приставать) + あう (взаимно).",
        "english_translation": "to associate with; to date (romantically); to socialize with\n\n> compound verb: 付く (attach) + あう (mutually).",
    },
    "斬る": {
        "russian_translation": "рубить (мечом); разрубать; убивать мечом\n\n> специфический кандзи для разрубания мечом, отличается от 切る.",
        "english_translation": "to slash; to cut down with a sword; to behead\n\n> specific kanji for cutting with a sword; distinct from 切る.",
    },
    "愛し合う": {
        "russian_translation": "любить друг друга; испытывать взаимную любовь\n\n> составной глагол: 愛する (любить) + あう (взаимно).",
        "english_translation": "to love each other; to be in mutual love\n\n> compound verb: 愛する (to love) + あう (mutually).",
    },
    "見れる": {
        "russian_translation": "может увидеть; в состоянии посмотреть\n\n> разговорная потенциальная форма глагола 見る (видеть).",
        "english_translation": "can see; able to watch\n\n> colloquial potential form of 見る (to see).",
    },
    "帰れる": {
        "russian_translation": "может вернуться; в состоянии уйти домой\n\n> потенциальная форма глагола 帰る (возвращаться).",
        "english_translation": "can return; able to go home\n\n> potential form of 帰る (to return).",
    },
    "モテる": {
        "russian_translation": "пользоваться популярностью (у противоположного пола); быть любимцем\n\n> сленговый глагол, обозначающий привлекательность для противоположного пола.",
        "english_translation": "to be popular (with the opposite sex); to be desirable\n\n> slang verb meaning attractiveness to the opposite sex.",
    },
    "歩ける": {
        "russian_translation": "может ходить; в состоянии идти пешком\n\n> потенциальная форма глагола 歩く (идти пешком).",
        "english_translation": "can walk; able to walk\n\n> potential form of 歩く (to walk).",
    },
    "貰える": {
        "russian_translation": "может получить; в состоянии взять\n\n> потенциальная форма глагола 貰う (получать).",
        "english_translation": "can receive; able to get\n\n> potential form of 貰う (to receive).",
    },
    "笑える": {
        "russian_translation": "может рассмеяться; можно посмеяться; вызывает смех\n\n> потенциальная форма глагола 笑う (смеяться).",
        "english_translation": "can laugh; able to laugh; laughable\n\n> potential form of 笑う (to laugh).",
    },
    "戻れる": {
        "russian_translation": "может вернуться; в состоянии возвратиться\n\n> потенциальная форма глагола 戻る (возвращаться).",
        "english_translation": "can return; able to go back\n\n> potential form of 戻る (to return).",
    },
    "守れる": {
        "russian_translation": "может защитить; в состоянии охранять; способен уберечь\n\n> потенциальная форма глагола 守る (защищать).",
        "english_translation": "can protect; able to defend; can preserve\n\n> potential form of 守る (to protect).",
    },
    "掴める": {
        "russian_translation": "может схватить; в состоянии ухватить; способен постичь\n\n> потенциальная форма глагола 掴む (хватать).",
        "english_translation": "can grasp; able to seize; can comprehend\n\n> potential form of 掴む (to grasp).",
    },
    "描ける": {
        "russian_translation": "может нарисовать; в состоянии изобразить; способен описать\n\n> потенциальная форма глагола 描く (рисовать, описывать).",
        "english_translation": "can draw; able to depict; can portray\n\n> potential form of 描く (to draw, to depict).",
    },
    "戦える": {
        "russian_translation": "может сражаться; в состоянии бороться\n\n> потенциальная форма глагола 戦う (сражаться).",
        "english_translation": "can fight; able to battle; able to combat\n\n> potential form of 戦う (to fight).",
    },
    "頑張れる": {
        "russian_translation": "может стараться; в состоянии выложиться; способен perseverировать\n\n> потенциальная форма глагола 頑張る (стараться, не сдаваться).",
        "english_translation": "can do one's best; able to persevere; can push through\n\n> potential form of 頑張る (to do one's best).",
    },
    "生まれ変わる": {
        "russian_translation": "переродиться; возродиться; переродиться в новом теле\n\n> составной глагол: 生まれる (рождаться) + 変わる (меняться).",
        "english_translation": "to be reborn; to be reincarnated; to start anew\n\n> compound verb: 生まれる (to be born) + 変わる (to change).",
    },
    "挿れる": {
        "russian_translation": "вставлять; помещать внутрь; вкладывать\n\n> глагол, обозначающий действие помещения чего-либо внутрь.",
        "english_translation": "to insert; to put in; to place inside\n\n> verb meaning the action of putting something inside.",
    },
    "殺る": {
        "russian_translation": "убить (груб.); прикончить; завалить (жарг.)\n\n> грубый/разговорный вариант глагола 殺す (убивать), записанный хираганой.",
        "english_translation": "to kill (rough); to finish off; to take down (slang)\n\n> rough/colloquial variant of 殺す (to kill) written in hiragana.",
    },
    "ぶっ殺す": {
        "russian_translation": "прикончить; убить (груб.); завалить\n\n> усилительный приставочный глагол: ぶっ (усилитель) + 殺す (убивать).",
        "english_translation": "to beat to death; to kill violently (vulgar)\n\n> emphatic verb: ぶっ (intensifier prefix) + 殺す (to kill).",
    },
    "勃つ": {
        "russian_translation": "вставать (о пенисе); иметь эрекцию (вульгарн.)\n\n> глагол, обозначающий эрекцию.",
        "english_translation": "to become erect (of penis); to have an erection (vulgar)\n\n> verb specifically denoting penile erection.",
    },
    "脈打つ": {
        "russian_translation": "пульсировать; биться (о пульсе); трепетать\n\n> составной глагол: 脈 (пульс) + 打つ (бить).",
        "english_translation": "to throb; to pulsate; to beat (with a pulse)\n\n> compound verb: 脈 (pulse) + 打つ (to beat/strike).",
    },
    "買える": {
        "russian_translation": "может купить; в состоянии приобрести\n\n> потенциальная форма глагола 買う (покупать).",
        "english_translation": "can buy; able to purchase\n\n> potential form of 買う (to buy).",
    },
    "好き勝手": {
        "russian_translation": "как заблагорассудится; эгоистично; по своему усмотрению\n\n> наречие/существительное, обозначающее действие по собственному желанию.",
        "english_translation": "as one pleases; selfishly; doing whatever one wants\n\n> adverb/noun meaning acting according to one's own desires.",
    },
    "思いっきり": {
        "russian_translation": "изо всех сил; что есть сил; от души\n\n> наречие, обозначающее действие с максимальным усилием.",
        "english_translation": "with all one's might; to the fullest; as hard as one can\n\n> adverb meaning acting with maximum effort.",
    },
    "過ごせる": {
        "russian_translation": "может провести (время); в состоянии прожить\n\n> потенциальная форма глагола 過ごす (проводить время).",
        "english_translation": "can spend (time); able to pass (time)\n\n> potential form of 過ごす (to spend time).",
    },
    "照れ": {
        "russian_translation": "смущение; стеснительность; застенчивость\n\n> существительное от глагола 照れる (смущаться).",
        "english_translation": "shyness; bashfulness; embarrassment\n\n> noun form of 照れる (to be shy/embarrassed).",
    },
    # === Real nouns ===
    "仕方ない": {
        "russian_translation": "ничего не поделаешь; неизбежный; не может быть иначе\n\n> часто используемое выражение для выражения покорности перед неизбежным.",
        "english_translation": "it can't be helped; unavoidable; nothing can be done\n\n> common expression for resignation to the inevitable.",
    },
    "やる気": {
        "russian_translation": "мотивация; энтузиазм; решимость; желание действовать\n\n> существительное, обозначающее внутренний стимул к действию.",
        "english_translation": "motivation; enthusiasm; drive; willingness to act\n\n> noun denoting internal drive or motivation.",
    },
    "兄妹": {
        "russian_translation": "брат и сестра; братья и сёстры\n\n> существительное, обозначающее siblings (брат и сестра).",
        "english_translation": "siblings (brother and sister); brothers and sisters\n\n> noun referring to siblings of mixed gender.",
    },
    "妖異": {
        "russian_translation": "сверхъестественное существо; ёкай; злой дух\n\n> существительное, обозначающее мистическое или демоническое существо.",
        "english_translation": "supernatural creature; yokai; evil spirit\n\n> noun referring to mystical or demonic beings.",
    },
    "存分": {
        "russian_translation": "в полной мере; досыта; сколько душе угодно\n\n> наречие/существительное, часто используется в составе 存分に.",
        "english_translation": "fully; to one's heart's content; abundantly\n\n> adverb/noun, commonly used in the form 存分に.",
    },
    "部室": {
        "russian_translation": "комната клуба; помещение кружка\n\n> сокращение от 部活動の部屋 (комната школьного клуба).",
        "english_translation": "club room; activity room\n\n> abbreviation of 部活動の部屋 (school club room).",
    },
    "暴走": {
        "russian_translation": "бесконтрольность; выход из-под контроля; безрассудный бег\n\n> существительное, обозначающее потерю контроля, часто о машинах или людях.",
        "english_translation": "running wild; going out of control; reckless behavior\n\n> noun denoting loss of control, often of vehicles or people.",
    },
    "偽者": {
        "russian_translation": "самозванец; подделка; фальшивка; обманщик\n\n> существительное, обозначающее того, кто выдаёт себя за другого.",
        "english_translation": "impostor; fake; fraud; phony\n\n> noun referring to someone who pretends to be someone else.",
    },
    "転生": {
        "russian_translation": "перерождение; реинкарнация; переселение души\n\n> существительное, часто используется в контексте аниме/манги о реинкарнации.",
        "english_translation": "reincarnation; rebirth; transmigration of souls\n\n> noun commonly used in anime/manga about reincarnation.",
    },
    "定番": {
        "russian_translation": "классика; стандарт; основной товар; неизменный хит\n\n> существительное, обозначающее что-то стандартное или классическое.",
        "english_translation": "standard; classic; staple; mainstay\n\n> noun denoting something standard or classic in a category.",
    },
    "甘えん坊": {
        "russian_translation": "избалованный ребёнок; любящий ласку; зависимый\n\n> существительное, описывающее человека, склонного к зависимости от других.",
        "english_translation": "spoiled child; pampered person; one who seeks affection\n\n> noun describing someone who relies on others' affection.",
    },
    "たこ焼き": {
        "russian_translation": "такояки (шарики из теста с осьминогом)\n\n> популярное японское уличное блюдо из Осаки.",
        "english_translation": "takoyaki (octopus balls)\n\n> popular Japanese street food from Osaka.",
    },
    "股間": {
        "russian_translation": "промежность; паховая область\n\n> анатомический термин, обозначающий область между ног.",
        "english_translation": "crotch; groin area\n\n> anatomical term for the area between the legs.",
    },
    "膣内": {
        "russian_translation": "внутри влагалища; влагалищный (анат.)\n\n> медицинский/анатомический термин.",
        "english_translation": "inside the vagina; vaginal (anatomical)\n\n> medical/anatomical term.",
    },
    "先っぽ": {
        "russian_translation": "кончик; край; наконечник; верхушка\n\n> разговорное существительное, обозначающее кончик или край чего-либо.",
        "english_translation": "tip; end; point; top\n\n> colloquial noun referring to the tip or end of something.",
    },
    "淫魔": {
        "russian_translation": "инкуб; суккуб; демон похоти\n\n> мифологическое существо, соблазняющее людей.",
        "english_translation": "incubus; succubus; lust demon\n\n> mythological creature that seduces people.",
    },
    "側室": {
        "russian_translation": "наложница; вторая жена (истор.)\n\n> исторический термин, обозначающий второстепенную жену или наложницу.",
        "english_translation": "concubine; secondary wife (historical)\n\n> historical term for a secondary wife or concubine.",
    },
    "先走り": {
        "russian_translation": "предэякулят; смазка (вульгарн.)\n\n> физиологический термин, обозначающий предсеменную жидкость.",
        "english_translation": "pre-ejaculate; pre-cum (vulgar)\n\n> physiological term for pre-seminal fluid.",
    },
    "肉まん": {
        "russian_translation": "мясной паровой пирожок (булочка с мясной начинкой)\n\n> китайское блюдо, популярное в Японии.",
        "english_translation": "meat bun; pork bun (steamed bun with meat filling)\n\n> Chinese dish popular in Japan.",
    },
    # === Slang / colloquial ===
    "エッチ": {
        "russian_translation": "развратный; непристойный; секс\n\n> сленговое слово от первой буквы hentai (変態). Может использоваться как прилагательное, существительное или глагол.",
        "english_translation": "lewd; perverted; sex; H\n\n> slang derived from the first letter of hentai (変態). Can be used as adjective, noun, or verb.",
    },
    "えっち": {
        "russian_translation": "развратный; непристойный; секс\n\n> хирагановое написание エッチ, имеет то же значение.",
        "english_translation": "lewd; perverted; sex; H\n\n> hiragana spelling of エッチ, same meaning.",
    },
    "マジ": {
        "russian_translation": "серьёзно; реально; правда?!\n\n> популярный сленг, обозначающий искренность или удивление.",
        "english_translation": "seriously; for real; really?!\n\n> popular slang expressing sincerity or surprise.",
    },
    "ラブコメ": {
        "russian_translation": "романтическая комедия; романтическая комедия (жанр аниме/манги)\n\n> сокращение от ラブコメディ (love comedy).",
        "english_translation": "romantic comedy; rom-com (anime/manga genre)\n\n> abbreviation of ラブコメディ (love comedy).",
    },
    "ヤバい": {
        "russian_translation": "опасный; ужасный; крутой; офигенный (сленг)\n\n>多功能ное сленговое слово, может означать как что-то плохое, так и что-то восхитительное.",
        "english_translation": "dangerous; terrible; awesome; amazing (slang)\n\n> versatile slang word that can mean something is either very bad or very good.",
    },
    "イチャイチャ": {
        "russian_translation": "флиртовать; миловаться; проявлять нежность на людях\n\n> ономатопоэтическое/мимическое слово для описания пары, проявляющей нежность.",
        "english_translation": "flirting; being lovey-dovey; PDA (public displays of affection)\n\n> mimetic word describing a couple being affectionate.",
    },
    "モヤモヤ": {
        "russian_translation": "туманность; неясное чувство; мрачное настроение; тревога\n\n> ономатопоэтическое слово для описания неясного, тягостного чувства.",
        "english_translation": "gloomy feeling; vague anxiety; frustration; foggy\n\n> onomatopoeic word for a vague, heavy feeling of unease.",
    },
    "もふもふ": {
        "russian_translation": "пушистый; мягкий и пушистый\n\n> ономатопоэтическое слово для описания чего-то мягкого и пушистого.",
        "english_translation": "fluffy; soft and fuzzy\n\n> onomatopoeic word describing something soft and fluffy.",
    },
    "バッチリ": {
        "russian_translation": "отлично; безупречно; на 100%; идеально\n\n> сленговое наречие, обозначающее полное совершенство.",
        "english_translation": "perfectly; flawlessly; spot-on; 100%\n\n> slang adverb meaning complete perfection.",
    },
    "テンション": {
        "russian_translation": "настроение; уровень энергии; возбуждённость\n\n> в японском сленге означает уровень энергии или воодушевления (не «напряжение»).",
        "english_translation": "mood; energy level; excitement level\n\n> in Japanese slang means energy or excitement level (not \"tension\").",
    },
    "ヘタレ": {
        "russian_translation": "слабак; неудачник; трус; никчёмный\n\n> сленговое существительное, обозначающее человека, неспособного на решительные действия.",
        "english_translation": "weakling; loser; coward; good-for-nothing\n\n> slang noun for someone unable to take decisive action.",
    },
    "キモい": {
        "russian_translation": "омерзительный; противный; жуткий (сленг)\n\n> сокращение от 気持ち悪い (неприятный). Очень распространённый молодёжный сленг.",
        "english_translation": "gross; disgusting; creepy (slang)\n\n> abbreviation of 気持ち悪い (unpleasant). Very common youth slang.",
    },
    "ロリコン": {
        "russian_translation": "лоликон; педофил (аттракция к девочкам)\n\n> сокращение от ロリータコンプレックス (комплекс Лолиты).",
        "english_translation": "lolicon; pedophile (attracted to young girls)\n\n> abbreviation of ロリータコンプレックス (Lolita complex).",
    },
    "ペロペロ": {
        "russian_translation": "лизание; облизывание\n\n> ономатопоэтическое слово, описывающее действие licking. Также может обозначать конфету-лизунец.",
        "english_translation": "licking; licking motion\n\n> onomatopoeic word describing licking. Can also refer to a lollipop.",
    },
    "カルボナーラ": {
        "russian_translation": "карбонара (итальянская паста)\n\n> заимствование из итальянского, популярное блюдо в Японии.",
        "english_translation": "carbonara (Italian pasta dish)\n\n> Italian loanword, popular dish in Japan.",
    },
    "今一": {
        "russian_translation": "не совсем; так себе; посредственный; не дотягивает\n\n> сокращение от 今一つ (не хватает одного). Означает «не совсем на уровне».",
        "english_translation": "not quite; so-so; mediocre; lacking something\n\n> abbreviation of 今一つ. Means \"not quite up to par.\"",
    },
    "ノーパン": {
        "russian_translation": "без трусов (сленг)\n\n> составное слово: ノー (нет) + パン (трусы, от англ. panties).",
        "english_translation": "no panties; not wearing underwear (slang)\n\n> compound word: ノー (no) + パン (panties).",
    },
    "ジンジン": {
        "russian_translation": "пульсирующая боль; покалывание; ноющее ощущение\n\n> ономатопоэтическое слово, описывающее ощущение пульсации или покалывания.",
        "english_translation": "throbbing; tingling; pulsating sensation\n\n> onomatopoeic word describing a throbbing or tingling feeling.",
    },
    # === Vulgar (explicit) ===
    "まんこ": {
        "russian_translation": "вагина (вульгарн.)\n\n> грубое сленговое слово, обозначающее женские половые органы.",
        "english_translation": "vagina; pussy (vulgar)\n\n> vulgar slang word for female genitalia.",
    },
    "ちんぽ": {
        "russian_translation": "пенис (вульгарн., сленг)\n\n> грубое сленговое слово, обозначающее мужские половые органы.",
        "english_translation": "penis; dick (vulgar, slang)\n\n> vulgar slang word for male genitalia.",
    },
    "チンポ": {
        "russian_translation": "пенис (вульгарн., сленг)\n\n> катакановое написание ちんぽ, имеет то же значение.",
        "english_translation": "penis; dick (vulgar, slang)\n\n> katakana spelling of ちんぽ, same meaning.",
    },
    "オナニー": {
        "russian_translation": "мастурбация (вульгарн.)\n\n> заимствование от немецкого Onanie или английского onanism.",
        "english_translation": "masturbation (vulgar)\n\n> loanword from German Onanie or English onanism.",
    },
    "ザーメン": {
        "russian_translation": "сперма (вульгарн.)\n\n> заимствование от немецкого Samen (семя).",
        "english_translation": "semen; cum (vulgar)\n\n> loanword from German Samen (seed).",
    },
    "クリトリス": {
        "russian_translation": "клитор (анат.)\n\n> анатомический термин.",
        "english_translation": "clitoris (anatomical)\n\n> anatomical term.",
    },
    "マンコ": {
        "russian_translation": "вагина (вульгарн.)\n\n> катакановое написание まんこ, имеет то же значение.",
        "english_translation": "vagina; pussy (vulgar)\n\n> katakana spelling of まんこ, same meaning.",
    },
    "てめぇ": {
        "russian_translation": "ты (груб.); ублюдок; ты, жалкий\n\n> грубое местоимение второго лица, используемое в агрессивной речи.",
        "english_translation": "you (derogatory); bastard; you piece of shit\n\n> rough second-person pronoun used in aggressive speech.",
    },
    "テメェ": {
        "russian_translation": "ты (груб.); ублюдок\n\n> катакановое написание てめぇ, имеет то же значение.",
        "english_translation": "you (derogatory); bastard\n\n> katakana spelling of てめぇ, same meaning.",
    },
    "みてぇ": {
        "russian_translation": "похожий на; подобный; как бы (груб. разн. みたい)\n\n> грубое/мужское произношение みたい (похожий на).",
        "english_translation": "like; similar to; seems like (rough form of みたい)\n\n> rough/masculine pronunciation of みたい (seems like).",
    },
    # === Other real words ===
    "儂": {
        "russian_translation": "я (местоимение старика)\n\n> устаревшее/характерное местоимение первого лица, используемое пожилыми мужчинами.",
        "english_translation": "I; me (old man's first-person pronoun)\n\n> archaic/character first-person pronoun used by elderly men.",
    },
    "胡散": {
        "russian_translation": "подозрительный; сомнительный; shady\n\n> часто используется в составе 胡散臭い (подозрительный).",
        "english_translation": "suspicious; dubious; shady\n\n> commonly used in the compound 胡散臭い (suspicious).",
    },
    "カーバンクル": {
        "russian_translation": "карбункул (мифическое существо); карбункул (драгоценный камень)\n\n> мифическое существо с драгоценным камнем на лбу, или название самого камня.",
        "english_translation": "carbuncle (mythical creature); carbuncle (gemstone)\n\n> mythical creature with a jewel on its forehead, or the gemstone itself.",
    },
}


def main() -> None:
    print(f"Reading: {CHUNK_PATH}")
    chunk_data: dict = json.loads(CHUNK_PATH.read_text(encoding="utf-8"))

    original_count = len(chunk_data)
    print(f"Existing word count: {original_count}")

    added = 0
    skipped_existing = 0

    for word, entry in NEW_WORDS.items():
        if word in chunk_data:
            print(f"  SKIP (already exists): {word}")
            skipped_existing += 1
        else:
            chunk_data[word] = entry
            added += 1

    print(f"\nAdded: {added}")
    print(f"Skipped (already existed): {skipped_existing}")
    print(f"New total: {len(chunk_data)}")

    CHUNK_PATH.write_text(
        json.dumps(chunk_data, ensure_ascii=False, indent=4) + "\n",
        encoding="utf-8",
    )
    print(f"\nWritten to: {CHUNK_PATH}")


if __name__ == "__main__":
    main()
