# WIC LIVE v4 - Game Patch Files

## Overview

Instead of running InstallShield patch executables with AutoIt automation,
we extract the patched files into zips and serve them from the backend.
The client downloads the zip, extracts over the game directory, and sets
the registry version.

## Patch: Full (1.0.0.0 -> 1.0.1.1)

For users with an unpatched game. Contains all files that are new or
modified compared to the v1.0.0.0 base install.

### New files (not present in v1.0.0.0)

| File | Size | MD5 |
|------|------|-----|
| laptopgamingvista.dll | 156,928 | 31BD0CA973A2D9D0DA419882FF8BEA7C |
| laptopgamingxp.dll | 156,928 | 52FCCF2DDCDA6FAC79010A8111B6C252 |
| madhoc.dll | 128,256 | 4E6DCBEA42977A3C517D0351146064AF |
| madhocvista.dll | 123,149 | A2707A1FC2F2F40D8EEF53F8D18B1EB8 |
| Massive_WorldInConflict.xml | 588 | C69B6698B6A8143134D5306EC30B0FA5 |
| wic20.sdf | 717,889 | C93C6813F24202AD12F57116E293C456 |
| wic21.sdf | 200 | ECCA737C90698DD423B096C08F132355 |
| wic25.sdf | 1,407,021 | D3EE9B94ACDFC2AC73265D8C4483B3A0 |
| wic30.sdf | 6,602,509 | 20709FD6DF349D908324BB05E9B6E41A |
| wic35.sdf | 35,305,463 | 1791FA786840AAFEC068C555E2EAD6CD |
| wic40.sdf | 9,566,081 | 7123EFBC797780E9D9E5520B036EE8FA |
| wic45.sdf | 12,813,588 | 4FD9594A53C6FBE42712DB58C56DF878 |
| wic47.sdf | 34,749 | B05F476B549CE4C6DD13FC36CA8EC887 |
| wic60.sdf | 915,943,689 | 0E385AA74B4C93DAF223E5717DFD5447 |
| wic65.sdf | 400,322,278 | 884762D13B7E6C3F0C60F772CBC2D878 |
| wicdata.uil | 3,187 | 92F2F0B67CD3EA25E8C9D077AA2D73BD |
| wicloc26.sdf | 73,074 | 9735023C2FEFC79697E0CB13B616D2EA |
| wicloc31.sdf | 65,391 | 59605827B09CD3EEEA87F15AB6B9D839 |
| wicloc36.sdf | 63,985 | 0093805DAF1F79E656DC3D423DE0604F |
| wicloc41.sdf | 71,394 | 5BB90A62C2776DEFC300A90D6972D9D8 |
| wicloc46.sdf | 63,321 | EE4E18C8E989DFE38E5C89BD126E95E9 |
| wicloc61.sdf | 450,299 | 9CF09327C9224D587F003F076DA1AB19 |
| wicloc62.sdf | 50,874,871 | 5FC0FD6E98907F5E5A37A38E694B84F3 |
| wicloc63.sdf | 273,507 | FA3820253E670BBF6B068729DDAB1FBE |
| wicloc66.sdf | 2,460 | D0A2FB7B28CE385EDFAF3963D2542D1D |
| wic_ds_banlist.txt | 720 | B5FB3996F35DEE6B437F3D67C6E1F80F |
| wic_ds_remote_howto.txt | 3,845 | 6C7C5BC3937D8BAC22A491E07525EA1D |
| speedtrees/wic_data/berlin_packedfiles.sdf | 10,562,763 | 6ED1E30979F94D6101B2FAAD480C0E56 |

### Modified files (different from v1.0.0.0)

| File | Size | MD5 |
|------|------|-----|
| wic.exe | 10,906,680 | BBBED9162C7DFC9CCEB999126667C506 |
| wic_online.exe | 10,484,792 | A343BDAEA973B43DCC6B941E1FB22483 |
| wic_ds.exe | 11,057,648 | BDE83369B57A8AFA4E982B808120AEDA |
| EULA.rtf | 22,136 | 4F70E8C52E639CC585503DEE19A13E7B |
| readme.txt | 49,600 | B8C3FB05FB7CA264C181DC03D2DF292A |
| WIC Registration.url | 83 | 9F25BF1F31E75E754CDAAB33AF854769 |
| wic_ds.ini | 9,366 | 36655F8D6E4D5049602391510A80EE45 |
| wic_ds_cycle.txt | 247 | C3988DEE3A87E1E57E2EF423369D97FE |
| speedtrees/wic_data/usfarmland_packedfiles.sdf | 5,213,415 | C411968E57276CE00B938B22000C5524 |

### Registry change
- `Version`: set to `1.0.1.1`

## Patch: P11 Only (1.0.1.0 -> 1.0.1.1)

For users who already have patch 10 applied.

### New files

| File | Size | MD5 |
|------|------|-----|
| wic65.sdf | 400,322,278 | 884762D13B7E6C3F0C60F772CBC2D878 |
| wicloc66.sdf | 2,460 | D0A2FB7B28CE385EDFAF3963D2542D1D |

### Modified files

| File | Size | MD5 |
|------|------|-----|
| wic.exe | 10,906,680 | BBBED9162C7DFC9CCEB999126667C506 |
| wic_online.exe | 10,484,792 | A343BDAEA973B43DCC6B941E1FB22483 |
| wic_ds.exe | 11,057,648 | BDE83369B57A8AFA4E982B808120AEDA |
| wicdata.uil | 3,187 | 92F2F0B67CD3EA25E8C9D077AA2D73BD |

### Registry change
- `Version`: set to `1.0.1.1`
