# Test Radarr metadata scrape

## Objective and initial state

Scrape a few movie records and posters through the local Rawrarr app so the user can inspect the resulting shared files. The app was healthy on `127.0.0.1:7297`; the host `/shared` bind resolved to `~/Library/Application Support/rawrarr/shared-data`. The metadata text and artwork directories existed but contained no files.

## Actions

- Used the already authenticated Rawrarr browser session to request poster routes for three existing Radarr movies: The Dead Girl (TMDb 13551), Fire in the Sky (15613), and 10 Cloverfield Lane (333371).
- Each route returned an image and wrote one text metadata ciphertext and one artwork ciphertext under the separate shared directories.

## Verification

- Confirmed all six files exist in the host shared-data directory.
- Confirmed each file starts with the age ciphertext header `age-encryption.org/v1` without decrypting or displaying private metadata.
- Sizes: text records 2,174, 2,318, and 2,287 bytes; artwork 415,121, 165,843, and 797,673 bytes.

## Final state and remaining work

The three sample records and posters are available in `shared-data/metadata/text/movies/` and `shared-data/metadata/artwork/movies/`. No further scraping was performed.
