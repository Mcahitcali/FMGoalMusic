# FM Goal Musics – Kullanım Kılavuzu

Bu kısa kılavuz, uygulamayı yükleme, ilk çalışma ve temel ayarlar hakkında bilgi verir.

## Kurulum
- DMG dosyasını açın (ör: `FM Goal Musics-arm64-YYYYMMDD.dmg` veya `FM Goal Musics-x86_64-YYYYMMDD.dmg`).
- `FM Goal Musics.app` simgesini DMG içindeki `Applications` kısayoluna sürükleyip bırakın.
- Kurulum tamamlandıktan sonra Uygulamalar klasöründen çalıştırın.

## İlk Çalıştırma (Gatekeeper)
Uygulama Apple tarafından imzalanıp noter tasdikli değildir. İlk açılışta uyarı görebilirsiniz:
- Çözüm 1: Uygulamaya sağ tıklayın → Open (Aç) → tekrar Open (Aç).
- Çözüm 2: Sistem Ayarları → Gizlilik ve Güvenlik → “Open Anyway (Yine de Aç)” seçeneği.

## Hızlı Başlangıç
1. Uygulamayı açın.
2. Ekran yakalama bölgesini belirleyin (maç metninin göründüğü alanı seçin).
3. Takımı seçin veya veri tabanından arayın.
4. `Start` ile tespiti başlatın. Gol algılandığında ses çalınır.

## OCR Ayarları
- `OCR Threshold`:
  - `0`: Otomatik Otsu eşikleme (önerilir). Görüntüye göre en uygun eşik değeri otomatik seçilir.
  - `1–255`: Manuel eşik değeri. Arka plan çok parlak/karanlık olduğunda ince ayar için kullanın.
- `Morph Open` (Gürültü azaltma):
  - Kapalı: Daha hızlı; temiz ekran görüntülerinde yeterlidir.
  - Açık: Biraz daha yavaş; gürültülü ekranlar için daha kararlı metin tespiti.

İpucu: Eşik çok düşükse harfler birbirine yapışabilir; çok yüksekse harfler silikleşir. Bozulma görürseniz değeri küçük adımlarla değiştirin veya 0’a (Otsu) dönün.

## Sık Karşılaşılan Sorunlar
- "Tessdata bulunamadı" hatası: Uygulama içindeki veri klasörü otomatik ayarlanır. Yine de sorun olursa uygulamayı Applications klasörüne taşıdığınızdan ve doğrudan oradan çalıştırdığınızdan emin olun.
- Gol metni yanlış tanınıyor:
  - Bölge seçiminde metnin tamamının kapsandığından emin olun.
  - `OCR Threshold`’u 0 yapıp deneyin.
  - Gerekirse `Morph Open`’ı açın.
- Uygulama açılamıyor: Gatekeeper talimatlarını uygulayın (sağ tık → Open veya "Open Anyway").

## Mimariler
- Apple Silicon (arm64) ve Intel (x86_64) için ayrı DMG’ler sağlanabilir.
- Kendi makinenize uygun olan DMG’yi kullanın. Emin değilseniz:  menüsü → Bu Mac Hakkında (About This Mac).

## Geri Bildirim
Sorun veya önerileriniz için DMG ile aynı sürüm bilgisini paylaşın (dosya adına gömülüdür) ve tekrar deneyebilmemiz için mümkünse ekran görüntüsü ekleyin.
