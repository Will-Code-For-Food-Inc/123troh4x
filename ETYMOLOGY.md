# Etymology

Names in this project are drawn from Japanese folklore and Shinto animism.

---

## tsukumogami (付喪神)

The spirit that awakens in a tool after 100 years of use. The canonical source is the
*Tsukumogami-ki* (付喪神記, "Record of Tool Kami"), a Muromachi-period prose tale. In it,
discarded household implements gain spirits on New Year's Eve, cause havoc, and get
subdued by a Buddhist monk.

It breaks into *tsukumo* and *gami* (the voiced form of *kami*). *Tsukumo* means something
long-used or aged. The numeric reading "99" comes from an old poetic convention: 100 years
is *momo* (百), so 99 is *momo* minus one stroke, which gets rendered *tsukumo*. The kanji
付喪 were applied later as ateji. Noriko T. Reider argues they echo the 4th-5th century
Chinese text *Sou Shen Ji*, whose *on'yomi* reading (*Fusoshin-ki*) is a homonym for
"Supplement to Soshin-ki." That's a deliberate pun embedded in the original text.

The compound first appears in the 9th-century *Ise Monogatari* (section 63), describing an
elderly woman's white hair. It wasn't attached to tool spirits until the Muromachi period.

The fit here is pretty obvious. `gami` runs inside containers full of decades-old
cross-compilers and retro SDKs. It's the awakened spirit of old tools.

**Sources:**

- Reider, Noriko T. "Animating Objects: *Tsukumogami ki* and the Medieval Illustration of
  Shingon Truth." *Japanese Journal of Religious Studies* 36, no. 2 (2009): 231-257.
  [JSTOR 40660967](https://www.jstor.org/stable/40660967)
- Komatsu, Kazuhiko. 器物の妖怪: 付喪神をめぐって. In *Nihon Yokai Ibunroku*, pp. 326-342.
  Kodansha Gakujutsu Bunko, 1994.
- Foster, Michael Dylan. *The Book of Yokai.* University of California Press, 2015.
  ISBN 978-0-520-27102-2.

---

## onmyoji (陰陽師)

A court diviner of the Heian period. The characters: *on* (陰, yin), *myo* (陽, yang),
*ji* (師, master). "Master of the Way of Yin and Yang." The associated practice,
onmyodo (陰陽道), appends *do* (道), the same character as in kendo, judo, aikido.

Chinese yin/yang cosmology reached Japan in the early 6th century CE. By the late 7th
century it had merged with Shinto, Taoism, Buddhism, and Confucianism into something
distinctly Japanese. The Taiho Code (701 CE) institutionalized it, giving onmyoji formal
bureaucratic roles at the Imperial Court: calendar keeping, divination, protecting the
capital from malevolent spirits.

The most famous practitioner is Abe no Seimei (921-1005 CE). His shikigami exploits are
documented in *Konjaku Monogatari*, *Okagami*, and *Genpei Josuiki*.

The role of onmyoji in this project is straightforward: it summons and commands spirits.
That's the MCP server.

**Sources:**

- Wikipedia: [Onmyoji](https://en.wikipedia.org/wiki/Onmy%C5%8Dji),
  [Onmyodo](https://en.wikipedia.org/wiki/Onmy%C5%8Dd%C5%8D)
- Yamayuandadu. "Shikigami and onmyodo through history: truth, fiction and everything in
  between." Primary texts cited include *Konjaku Monogatari*, *Okagami*, *Hoki Naiden*,
  *Sakeiki*, *Pillow Book*, *Shinsarugakuki*. Scholars cited include Faure, Pang, Shigeta,
  Hayashi, Triplett, Ueno.
  <https://yamayuandadu.tumblr.com/post/740891749838471168/>

---

## shikigami (式神)

A spirit entity bound and directed by an onmyoji. *Shiki* (式) means ritual formula or
binding rite. *Gami* is the spirit. A kami conscripted through ceremonial working.

What a shikigami actually is was contested even in the Heian period. Some texts describe
invisible servants. Others depict paper dolls animated by the onmyoji's power. Others
place them as subordinate spirits residing inside the practitioner. The *Hoki Naiden*
(traditionally attributed to Abe no Seimei but dated by modern scholars to the 14th
century) is the primary doctrinal source.

Shikigami was an early candidate for the in-container runner name. It didn't stick.
Tsukumogami fits better because the spirit isn't summoned fresh each time; it lives inside
the toolchain permanently.

---

## kami (神) and rendaku

*Kami* (神) is the Shinto term for divine spirit. The etymology is genuinely contested.
One reading derives it from *kagami* (鏡, mirror), which are sacred objects in Shinto.
Another traces it to *kamu*, an archaic verb for destructive natural force. Motoori
Norinaga (1730-1801) defined kami as "anything possessing an extraordinary quality that
inspires awe," and explicitly refused to limit it to benevolent or anthropomorphic entities.
That definition has held up.

The character 神 comes from Chinese *shen*, but the native Japanese reading *kami*
predates the borrowing.

**Why the binary is `gami` and not `kami`:** Rendaku (連濁, "sequential voicing") is a
Japanese morphophonological rule where the initial consonant of a non-initial element in a
compound gets voiced. *k* becomes *g*. So kami becomes gami in tsukumo**gami**,
shiki**gami**, and *megami* (女神, goddess: *me* + *kami*). Rendaku is attested in Old
Japanese from the 8th century CE. It's subject to Lyman's Law: voicing is blocked if the
second element already contains a voiced obstruent, which is why some compounds don't
trigger it.

*Gami* isn't a standalone lexeme in modern Japanese (you'd say *kami* in isolation), so
the binary name doesn't conflict with anything.

**Sources:**

- Wikipedia: [Rendaku](https://en.wikipedia.org/wiki/Rendaku)
- Vance, Timothy J. *The Sounds of Japanese.* Cambridge University Press, 2008.
  ISBN 978-0-521-61438-1.
- Tofugu. "Rendaku: Why Hito-Bito isn't Hito-Hito."
  <https://www.tofugu.com/japanese/rendaku/>
- "Sequential Voicing (Rendaku) in Japanese." *Language Miscellany*, 2022.
  <https://languagemiscellany.com/2022/03/sequential-voicing-rendaku-in-japanese/>
