# Extracted text of `RYLR896_EN.pdf`

> Auto-generated with `pdftotext -layout`. Line-art, images, and image-based pages do not survive extraction; verify anything load-bearing against the source PDF.

```text
                             01-Nov-2021 56312E37




RYLR896
UART Interface
868/915MHz LoRa®
Antenna Transceiver Module



Datasheet
                                                                                   RYLR896
                                                  868/915MHz LoRa Antenna Transceiver Module




PRODUCT DESCRIPTION
The RYLR896 transceiver module feature the LoRa long range modem that provides ultra-long range
spread spectrum communication and high interference immunity whilst minimising current consumption.
The RYLR896 is certified by NCC and FCC.


FEATURES
•   Semtech SX1276 Engine
•   Excellent blocking immunity
•   Low receive current
•   High sensitivity
•   Control easily by AT commands
•   127 dB Dynamic Range RSSI
•   Designed with integrated antenna
•   AES128 Data encryption


APPLICATIONS
•   IoT Applications
•   Mobile Equipment
•   Home Security
•   Industrial Monitoring and Control Equipment
•   Car Alarm


CERTIFICATION
•   FCC

•   NCC




Copyright © 2019, REYAX TECHNOLOGY CO., LTD.                                                    2
                                                                                        RYLR896
                                                       868/915MHz LoRa Antenna Transceiver Module




PIN DESCRIPTION




             Pin          Name                 I/O                      Condition
         1         VDD                  I            Power Supply
         2         NRST                 I
                                                     RESET(Active Low)100KΩ Internal pull up,

                                                     Pull down at least 100ms
         3         RXD                  I            UART Data Input
         4         TXD                  O            UART Data Output
         5         NC                   -
         6         GND                  -            Ground




Copyright © 2019, REYAX TECHNOLOGY CO., LTD.                                                        3
                                                                                             RYLR896
                                                         868/915MHz LoRa Antenna Transceiver Module




BLOCK DIAGRAM


                                     RYLR896




SPECIFICATION
             Item                   Min.       Typical          Max.       Unit              Condition
 VDD Power Supply                     2          3.3             3.6         V     VDD
 RF Output Power Range               -4                          15        dBm
 Filter insertion loss                1          2                3         dB
 RF Sensitivity                     -148                                   dBm
 RF Input Level                                                  10        dBm
 Frequency Range                    862        868/915          1020       MHz
 Frequency Accuracy                              ±2                        ppm
 Communication Range                             4.5             15         KM     Depend on RF parameter
                                                                                   & environment.
 Transmit Current                               49.7                        mA     RFOP = +14dBm
 Receive Current                                16.5                        mA     AT+MODE=0
 Sleep Current                                   0.5                        uA     AT+MODE=1
 Baud rate                          300        115200          115200       bps    8, N, 1
 Digital Input Level High        0.7*VDD                        VDD          V     VIH
 Digital Input Level Low              0                       0.3*VDD        V     VIL
 Digital Output Level High          0.9                         VDD          V     VOH
 Digital Output Level Low                                        0.1         V     VOL
 Cycling (erase / write)                        300                          K    Cycles
 EEPROM data memory
 Weight                                         3.07                         g
 Operating Temperature              -40          25              +85        ˚C



Copyright © 2019, REYAX TECHNOLOGY CO., LTD.                                                                4
                                                                                RYLR896
                                               868/915MHz LoRa Antenna Transceiver Module




DIMENSIONS




                   Unit : mm




Copyright © 2019, REYAX TECHNOLOGY CO., LTD.                                                5
                                                                                RYLR896
                                               868/915MHz LoRa Antenna Transceiver Module




                         Unit : mm




Copyright © 2019, REYAX TECHNOLOGY CO., LTD.                                                6
                                                                                                             RYLR896
                                                                  868/915MHz LoRa Antenna Transceiver Module




CERTIFICATION INFORMATION
Taiwan NCC Statement 低功率電波輻射性電機管理辦法:

第十二條 經型式認證合格之低功率射頻電機，非經許可，公司、商號或使用者均不得擅自變更頻率、加大功率或變更原設
           計之特性及功能。

第十四條 低功率射頻電機之使用不得影響飛航安全及干擾合法通信；經發現有干擾現象時，應立即停用，並改善至無干擾
           時方得繼續使用。前項合法通信，指依電信法規定作業之無線電通信。低功率射頻電機須忍受合法通信或工業、
           科學及醫療用電波輻射性電機設備之干擾。



          CCAN18LP0920T8

FCC Statement:
This device complies with part 15 of the FCC Rules. Operation is subject to the following two conditions:
(1) This device may not cause harmful interference, and
(2) this device must accept any interference received, including interference that may cause undesired operation.

NOTE: This equipment has been tested and found to comply with the limits for a Class B digital device, pursuant to part 15 of the FCC
Rules. These limits are designed to provide reasonable protection against harmful interference in a residential installation.
This equipment generates, uses and can radiate radio frequency energy and, if not installed and used in accordance with the
instructions, may cause harmful interference to radio communications. However, there is no guarantee that interference will not
occur in a particular installation.
If this equipment does cause harmful interference to radio or television reception, which can be determined by turning the
equipment off and on, the user is encouraged to try to correct the interference by one or more of the following measures:
—Reorient or relocate the receiving antenna.
—Increase the separation between the equipment and receiver.
—Connect the equipment into an outlet on a circuit different from that to which the receiver is connected.
—Consult the dealer or an experienced radio/TV technician for help.
Changes or modifications not expressly approved by the party responsible for compliance could void the user’s authority to operate
the equipment.

LABEL OF THE END PRODUCT:
The final end product must be labeled in a visible area with the following " Contains TX FCC ID : QLY-RYLR896 ". If the size of the end
product is larger than 8x10cm, then the following FCC part 15.19 statement has to also be available on the label: This device complies
with Part 15 of FCC rules. Operation is subject to the following two conditions: (1) this device may not cause harmful interference
and (2) this device must accept any interference received, including interference that may cause undesired operation.



           QLYRYLR896




                                                                                   E-mail: sales@reyax.com
                                                                                   Website: http://reyax.com




Copyright © 2019, REYAX TECHNOLOGY CO., LTD.                                                                                         7
```
