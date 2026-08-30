# Extracted text of `1893_datasheet.pdf`

> Auto-generated with `pdftotext -layout`. Line-art, images, and image-based pages do not survive extraction; verify anything load-bearing against the source PDF.

```text
Freescale Semiconductor                                                                     Document Number: MPL3115A2
Data Sheet:Technical Data                                                                               Rev 3.0, 12/2013
An Energy Efficient Solution by Freescale




Xtrinsic MPL3115A2 I2C Precision                                                              MPL3115A2
Altimeter                                                                                            50 to 110 kPa
   The Xtrinsic MPL3115A2 employs a MEMS pressure sensor with an I2C
interface to provide accurate Pressure/Altitude and Temperature data. The
sensor outputs are digitized by a high resolution 24-bit ADC. Internal processing
removes compensation tasks from the host MCU system. Multiple user-
programmable, power saving, interrupt and autonomous data acquisition modes
are available, including programmed acquisition cycle timing, and poll-only
modes. Typical active supply current is 40 μA per measurement-second for a
stable 10 cm output resolution.
   The MPL3115A2 is offered in a 5 mm x 3 mm x 1.1 mm LGA package and
specified for operation from -40 °C to 85 °C. Package is surface mount with a
stainless steel lid and is RoHS compliant.

Features                                                                                          LGA PACKAGE
                                                                                             5.0 mm x 3.0 mm x 1.1 mm
•   1.95 V to 3.6 V Supply Voltage, internally regulated by LDO
•   1.6 V to 3.6 V Digital Interface Supply Voltage
•   Fully Compensated internally
•   Direct Reading, Compensated                                                                        Top View
       – Pressure: 20-bit measurement (Pascals)
       – Altitude: 20-bit measurement (meters)                                                VDD      1       8     SCL

       – Temperature: 12-bit measurement (degrees Celsius)
                                                                                              CAP      2       7     SDA
•   Programmable Events
•   Autonomous Data Acquisition                                                              GND       3       6     INT1
•   Resolution down to 0.1 m
                                                                                             VDDIO     4       5     INT2
•   32-Sample FIFO
•   Ability to log data up to 12 days using the FIFO
•   1 second to 9 hour data acquisition rate                                                     Pin Connections
•   I2C digital output interface (operates up to 400 kHz)

Application Examples
•   High Accuracy Altimetry
•   Smartphones/Tablets
•   Personal Electronics Altimetry
•   GPS Dead Reckoning
•   GPS Enhancement for Emergency Services
•   Map Assist, Navigation
•   Weather Station Equipment

                                                   ORDERING INFORMATION
                                                              # of Ports                    Pressure Type                    Digital
    Device Name      Package Options      Case No.
                                                      None     Single      Dual     Gauge    Differential   Absolute        Interface
MPL3115A2                  Tray             2153         •                                                     •                •
MPL3115A2R1         Tape & Reel (1000)      2153         •                                                     •                •




© 2011-2013 Freescale Semiconductor, Inc. All rights reserved.
Contents
1   Block Diagram and Pin Descriptions . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 3
2   Mechanical and Electrical Specifications. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 5
    2.1 Mechanical Characteristics . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 5
3   Electrical Characteristics . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 6
    3.1 I2C Interface Characteristics . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 7
    3.2 Absolute Maximum Ratings . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 9
4   Quick Start Setup . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 10
5   Terminology . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 12
    5.1 Resolution . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 12
    5.2 Accuracy . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 12
    5.3 Pressure/Altitude. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 12
    5.4 Power Modes of Operation . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 13
6   Functionality . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 13
    6.1 Factory Calibration . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 14
    6.2 Barometer/Altimeter Function . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 14
    6.3 Temperature Function. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 14
    6.4 Autonomous Data Acquisition. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 14
    6.5 FIFO . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 14
    6.6 External Interrupts. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 14
    6.7 Pressure/Altitude and Temperature Delta. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 15
    6.8 Min/Max Data Value Storage . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 15
    6.9 Digital Interface . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 15
7   Register Descriptions. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 17
    7.1 Sensor Status . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 19
    7.2 Device ID . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 23
    7.3 FIFO Setup . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 23
    7.4 Time Delay Register . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 25
    7.5 System Mode Register . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 25
    7.6 System Interrupt Status . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 25
    7.7 Sensor Data . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 26
    7.8 Barometric Pressure Input . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 27
    7.9 Pressure/Altitude Target . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 27
    7.10 Temperature Target . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 27
    7.11 Pressure/Altitude Window. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 28
    7.12 Temperature Window . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 28
    7.13 Minimum Pressure . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 28
    7.14 Maximum Pressure . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 29
    7.15 Minimum Temperature . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 29
    7.16 Maximum Temperature. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 30
    7.17 Control Registers . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 30
    7.18 Offset Correction. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 35
8   Soldering/Landing Pad Information . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 37
9   Tape and Reel Specifications . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 40




Related Documentation
The MPL3115A2 device features and operations are described in a variety of reference manuals, user guides, and application
notes. To find the most-current versions of these documents:
    1.     Go to the Freescale homepage at:
                                                                        http://www.freescale.com/
    2.     In the Keyword search box at the top of the page, enter the device number MPL3115A2.
    3.     In the Refine Your Result pane on the left, click on the Documentation link.

MPL3115A2

                                                                                                                                                              Sensors
2                                                                                                                                          Freescale Semiconductor, Inc.
1       Block Diagram and Pin Descriptions


                                                                                                         VVdd
                                                                                                           DD
                                                                                         Reference
                                                                                         Reference       VDDIO
                                                                                             +           VddIO
                                                                                             +
                                                                                         Regulator
                                                                                         Regulator       CAP
                                                                                                         CAP

                                                                                                         SDA
                                                                                                         SDA

                                                                                                         SCL
                                                                                                         SCL
                                   Sense
           Pressure                Amp
            Sensor                                                                      Digital Signal
                                                      MUX                ADC           Processing and
                                                                                           Control       INT1
                                                                                                          INT1
                                        Temp
                                                                                                         INT2
                                                                                                          INT2



                                                                Clock oscillator         Trim Logic
                                                              Clock Oscillator           Trim Logic



                                                  Figure 1. Block Diagram


                 Table 1. Pin Descriptions
                   Pin          Name                                     Function
                    1           VDD      VDD Power Supply Connection (1.95-3.6 V)

                    2           CAP      External Capacitor

                    3           GND      Ground

                    4           VDDIO    Digital Interface Power Supply (1.62-3.6 V)

                    5           INT2     Pressure Interrupt 2

                    6           INT1     Pressure Interrupt 1

                    7           SDA      I2C Serial Data

                    8           SCL      I2C Serial Clock




                                                                                                         MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                    3
                                      Vdd
                                       DD

                                       100nF    10μF                               SCL
                                                         1                  8




                                                                                   SDA
                                                         2                 7



                                               100nF
                                                                                   INT1
                                                         3                 6




                                               VDDIO
                                               VddIO                               INT2
                                                         4                 5




                                                   Figure 2. Pin Connections


   The device power is supplied through the VDD line. Power supply decoupling capacitors (100 nF ceramic plus 10 μF bulk or
10 μF ceramic) should be placed as near as possible to pin 1 of the device. A second 100 nF capacitor is used to bypass the
internal regulator. The functions, threshold and the timing of the interrupt pins (INT1 and INT2) are user programmable through
the I2C interface.




MPL3115A2

                                                                                                                   Sensors
4                                                                                               Freescale Semiconductor, Inc.
2         Mechanical and Electrical Specifications
2.1       Mechanical Characteristics
Table 2. Mechanical Characteristics @ VDD = 2.5 V, T = 25 °C unless otherwise noted(1)
 Ref    Symbol                  Parameter                          Test Conditions                 Min          Typ          Max          Unit
 Pressure Sensor
                                                                   Calibrated Range                50                        110          kPa
    1     PFS     Measurement Range
                                                                  Operational Range                20                        110          kPa

                                                                   1x Oversample(2)                             19                      Pa RMS
    2             Pressure Reading Noise
                                                                                     (2)                        1.5                     Pa RMS
                                                                  128x Oversample

                                                                     50 to 110 kPa
                                                                                                   -0.4                      0.4
                                                                   over 0 °C to 50 °C
    3             Pressure Absolute Accuracy                                                                                              kPa
                                                                     50 to 110 kPa
                                                                                                               ±0.4
                                                                  over -10 °C to 70 °C

                                                                Relative accuracy during
                                                               pressure change between
                                                             70 to 110 kPa at any constant                     ±0.05
                                                                 temperature between
                                                                    -10 °C to 50 °C
    4             Pressure Relative Accuracy                                                                                              kPa
                                                                Relative accuracy during
                                                            changing temperature between
                                                            -10 °C to 50 °C at any constant                    ±0.1
                                                              pressure between 50 kPa to
                                                                        110 kPa

                                                                   Barometer Mode                 0.25          1.5                        Pa
    5             Pressure/Altitude Resolution(3)(4)(5)
                                                                    Altimeter Mode               0.0625         0.3                        m

                                                                     OST(6) Mode                                100                        Hz
    6             Output Data Rate
                                                                      FIFO Mode                                               1            Hz

 Temperature Sensor
    7     TFS     Measurement Range                                                                -40                       +85           °C

                                                                        @ 25 °C                                 ±1
    8             Temperature Accuracy                                                                                                     °C
                                                               Over Temperature Range                           ±3

    9     TOP     Operating Temperature Range                                                      -40                       +85           °C



 10               Board Mount Drift                                After solder reflow                         ±0.15                      kPa

 11               Long Term Drift                               After a period of 1 year                       ±0.1                       kPa

1. Measured at 25 °C, over 50 kPa to 110 kPa.
2. Oversample (OSR) modes internally combine and average samples to reduce noise.
3. Smallest bit change in register represents minimum value change in Pascals or meters. Typical resolution to signify change in altitude is 0.3 m.
4. At 128x Oversample Ratio.
5. Reference pressure = 101.325 kPa (Sea Level).
6. OST = One Shot Mode.




                                                                                                                                   MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                                    5
3           Electrical Characteristics
Table 3. Electrical Characteristics @ VDD = 2.5 V, T = 25 °C unless otherwise noted.
 Ref      Symbol                    Parameter                     Test Conditions            Min         Typ        Max         Unit
     1    VDDIO      I/O Supply Voltage                                                      1.62        1.8         3.6         V

     2     VDD       Operating Supply Voltage                                                1.95        2.5         3.6         V

                                                                 Highest Speed Mode
     3                                                                                                   8.5
                                                                   Oversample = 1

                                                                   Standard Mode
     4      IDD      Integrated Current 1 update per second                                              40                     µA
                                                                  Oversample = 16

                                                                High Resolution Mode
     5                                                                                                   265
                                                                 Oversample = 128

                     Max Current during Acquisition and
     6    IDDMAX                                                  During Acquisition                      2                     mA
                     Conversion

                                                              STANDBY Mode selected
     7    IDDSTBY Supply Current Drain in STANDBY Mode                                                    2                     µA
                                                                   SBYB = 0

                     Digital High Level Input Voltage
     8     VIH                                                                               0.75                              VDDIO
                         SCL, SDA

                     Digital Low Level Input Voltage
     9     VIL                                                                                                       0.3       VDDIO
                         SCL, SDA

                     High Level Output Voltage
     10    VOH                                                       IO = 500 µA             0.9                               VDDIO
                         INT1, INT2

                     Low Level Output Voltage
     11    VOL                                                       IO = 500 µA                                     0.1       VDDIO
                        INT1, INT2

                     Low Level Output Voltage
     12   VOLS                                                       IO = 500 µA                                     0.1       VDDIO
                        SDA

                                                               Highest Speed Mode(2)                                 60
     14    TON                   (1)                                                                                            ms
                     Turn-on time
                                                              Highest Resolution Mode(3)                            1000

     16    TOP       Operating Temperature Range                                             -40         25         +85         °C
     2C Addressing
 I
MPL3115A2 uses 7-bit addressing and does not acknowledge general call address 000 0000. Slave address has been set to 0x60 or 110 0000.
8-bit read is 0xC1, 8-bit write is 0xC0

1. Time to obtain valid data from “STANDBY” mode to “ACTIVE” mode.
2. High Speed mode is achieved by setting the oversample rate of 1x.
3. High Resolution mode is achieved by setting the oversample to 128x.




MPL3115A2

                                                                                                                         Sensors
6                                                                                                     Freescale Semiconductor, Inc.
3.1       I2C Interface Characteristics
Table 4. I2C Slave Timing Values(1)

                                                                                                           I 2C
   Ref       Symbol                          Parameter                                                                                       Unit
                                                                                          Condition                   Min              Max
      1        fSCL     SCL Clock Frequency                                      Pull-up = 1 kΩ, Cb = 400 pF           0               400   KHz

      2        fSCL     SCL Clock Frequency                                      Pull-up = 1 kΩ, Cb = 20 pF            0                4    MHz

      3        tBUF     Bus free time between STOP and START condition                                                1.3                     µs

      4      tHD;STA    Repeated START Hold Time                                                                      0.6                     µs

      5      tSU;STA    Repeated START Setup Time                                                                     0.6                     µs

      6      tSU;STO    STOP Condition Setup Time                                                                     0.6                     µs

      7      tHD;DAT    SDA Data Hold Time(2)                                                                         50(3)            (4)
                                                                                                                                              ns
                                                                                                                           (5)
      8      tSU;DAT    SDA Setup Time                                                                               100                      ns

      9        tLOW     SCL Clock Low Time                                                                            1.3                     µs

    10         tHIGH    SCL Clock High Time                                                                           0.6                     µs
                                                                                                                                 (6)
    11           tr     SDA and SCL Rise Time                                                                     20 + 0.1Cb           300    ns

    12           tf     SDA and SCL Fall Time (3)(6)(7)(8)                                                        20 + 0.1Cb(6)        300    ns

                        Pulse width of spikes that are suppressed by internal
    13          tSP                                                                                                                    50     ns
                        input filter

1. All values referred to VIH(min) and VIL(max) levels.
2. tHD;DAT is the data hold time that is measured from the falling edge of SCL, applies to data in transmission and the acknowledge.
3. The device must internally provide a hold time of at least 300 ns for the SDA signal (with respect to the VIH(min) of the SCL signal) to bridge
   the undefined region of the falling edge of SCL.
4. The maximum tHD;DAT must be less than the maximum of tVD;DAT or tVD;ACK by a transition time. This device does not stretch the LOW period
   (tLOW) of the SCL signal.
5. A fast mode I2C device can be used in a standard mode I2C system, but the requirement tSU;DAT 250 ns must then be met. This will
   automatically be the case if the device does not stretch the LOW period of the SCL signal. If such a device does stretch the LOW period of
   the SCL signal, it must output the next data bit to the SDA line tr(max) + tSU;DAT = 1000 + 250 = 1250 ns (according to the standard mode I2C
   specification) before the SCL line is released. Also the acknowledge timing must meet this set-up time.
6. Cb = Total capacitance of one bus line in pF.
7. The maximum tf for the SDA and SCL bus lines is specified at 300 ns. The maximum fall time for the SDA output stage tf is specified at 250 ns.
   This allows series protection resistors to be connected in between the SDA and the SCL pins and the SDA/SCL bus lines without exceeding
   the maximum specified tf.
8. In Fast Mode Plus, fall time is specified the same for both output stage and bus timing. If series resistors are used, designers should allow for
   this when considering bus timing.




                                                    Figure 3. I2C Slave Timing Diagram

                                                                                                                                        MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                                      7
                MSB                                  LSB                          MSB                                        LSB
      SCL        1    2     3     4     5   6   7     8     9                      1    2       3    4     5      6    7      8     9




      SDA       AD7 AD6 AD5 AD4 AD3 AD2 AD1 R/W                        XXX        D7    D6    D5     D4    D3     D2   D1     D0


        Start             Calling Address           Read/   Ack                                     Data Byte                       No     Stop
       Signal                                       Write   Bit                                                                     Ack   Signal
                                                                                                                                    Bit


                MSB                                  LSB                          MSB                                        LSB
      SCL        1    2     3     4     5   6   7      8    9                      1    2       3    4     5      6    7      8      9




      SDA       AD7 AD6 AD5 AD4 AD3 AD2 AD1 R/W                   XX              AD7 AD6 AD5 AD4 AD3 AD2 AD1 R/W


        Start             Calling Address           Read/   Ack        Repeated             New Calling Address             Read/   No Stop
       Signal                                       Write   Bit          Start                                              Write   Ack Signal
                                                                        Signal                                                      Bit


                                            Figure 4. I2C Bus Transmission Signals




MPL3115A2

                                                                                                                                      Sensors
8                                                                                                                  Freescale Semiconductor, Inc.
3.2      Absolute Maximum Ratings
   Stresses above those listed as “absolute maximum ratings” may cause permanent damage to the device. This is a stress rating
only and functional operation of the device under these conditions is not implied. Exposure to maximum rating conditions for ex-
tended periods may affect device reliability.

Table 5. Maximum Ratings
                                Characteristic                                  Symbol                     Value               Unit
Maximum Applied Pressure                                                          Pmax                      500                kPa

Supply Voltage                                                                    VDD                    -0.3 to 3.6            V

Interface Supply Voltage                                                         VDDIO                   -0.3 to 3.6            V

Input Voltage on any Control Pin (SCL, SDA)                                       VIN                -0.3 to VDDIO + 0.3        V

Operating Temperature Range                                                       TOP                    -40 to +85            °C

Storage Temperature Range                                                         TSTG                  -40 to +125            °C

Table 6. ESD And Latch-up Protection Characteristics
                                   Rating                                       Symbol                     Value               Unit
Human Body Model                                                                  HBM                      ±2000                V

Machine Model                                                                     MM                       ±200                 V

Charge Device Model                                                               CDM                      ±500                 V

Latch-up Current at T = 85 °C                                                      —                       ±100                mA

       This is a mechanical shock sensitive device, improper handling can cause permanent damage to the part or
       cause the part to otherwise fail.

        This is an ESD sensitive device, improper handling can cause permanent damage to the part.




                                                                                                                           MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                         9
4        Quick Start Setup
   To set up the part in Altimeter mode, you may select your data retrieval method between polling (no FIFO), interrupt (no FIFO)
or with the FIFO.
   The flow chart in Figure 5 describes the setup for polling or interrupt with an OSR of 128. Figure 6 describes the FIFO setup
with an OSR of 128.
  See the application note titled Data Manipulation and Basic Settings of the MPL3115A2 Command Line Interface (document
AN4519).


                        /* I2C Address is 0xC0 */
                        SlaveAddressIIC = 0xC0




                 /* Set to Altimeter with an OSR = 128 */                                   /* Set INT to Active Low Open Drain */
              IIC_RegWrite(SlaveAddressIIC, 0x26, 0xB8);                                 IIC_RegWrite(SlaveAddressIIC, 0x28, 0x11);




               /* Enable Data Flags in PT_DATA_CFG */
             IIC_RegWrite(SlaveAddressIIC, 0x13, 0x07);                                          /* Enable DRDY Interrupt */
                                                                                          IIC_RegWrite(SlaveAddressIIC, 0x29, 0x80);




                                                                                                        /* Set Active */
                                                                                          IIC_RegWrite(SlaveAddressIIC, 0x26, 0xB9);




                                                        Interrupt
                           Poll data or use
                           interrupt INT2                                                            Check for interrupt          No
                                                                                                       on pin INT2


                                                                    Service interrupt
                                       Polling                         and clear
                                                                                                                  Yes

                            /* Set Active */
             IIC _RegWrite(SlaveAddressIIC, 0x26, 0xB9);                                       /* Read INT_SOURCE */
                                                                                    INT_S = IIC _RegRead(SlaveAddressIIC, 0x12);


                     /* Read STATUS Register */
             STA = IIC _RegRead(SlaveAddressIIC, 0x00);




                                                                                    No
                                                                                                    /* Is SRC_DRDY Set */
                          /* Is Data Ready */             No                                      (INT_S & 0x80) == TRUE
                        (STA & 0x08) == TRUE




                             Yes
                                                                                                                 Yes

                      /* Read OUT_P and OUT_T */                                                 /* Read OUT_P and OUT_T */
                   /* This clears the DRDY Interrupt */                                       /* This clears the DRDY Interrupt */
             OUT_P_MSB = IIC_RegRead(SlaveAddressIIC, 0x01);                            OUT_P_MSB = IIC_RegRead(SlaveAddressIIC, 0x01);
             OUT_P_CSB = IIC_RegRead(SlaveAddressIIC, 0x02);                            OUT_P_CSB = IIC_RegRead(SlaveAddressIIC, 0x02);
             OUT_P_LSB = IIC_RegRead(SlaveAddressIIC, 0x03);                            OUT_P_LSB = IIC_RegRead(SlaveAddressIIC, 0x03);
             OUT_T_MSB = IIC_RegRead(SlaveAddressIIC, 0x04);                            OUT_T_MSB = IIC_RegRead(SlaveAddressIIC, 0x04);
             OUT_T_LSB = IIC_RegRead(SlaveAddressIIC, 0x05);                            OUT_T_LSB = IIC_RegRead(SlaveAddressIIC, 0x05);
                                                                                                         Clear Interrupt

                                                 Figure 5. Polling or Interrupt - No FIFO

MPL3115A2

                                                                                                                                     Sensors
10                                                                                                                Freescale Semiconductor, Inc.
                  /* I2C Address is 0xC0 */
                  SlaveAddressIIC = 0xC0
                                                                                       /* Set INT to Active Low Open Drain */
                                                                                   IIC _RegWrite(SlaveAddressIIC, 0x28, 0x11);

           /* Set to Altimeter with an OSR = 128 */
        IIC_RegWrite(SlaveAddressIIC, 0x26, 0xB8);

                                                                                            /* Enable FIFO Interrupt */
                                                                                    IIC _RegWrite(SlaveAddressIIC, 0x29, 0x40);

        /* Enable Data Flags in PT_DATA_CFG */
      IIC_RegWrite(SlaveAddressIIC, 0x13, 0x07);


                                                                                                  /* Set Active */
                                                                                    IIC_RegWrite(SlaveAddressIIC, 0x26, 0xB9);
          /* Set INT to Active Low Open Drain */
       IIC_RegWrite(SlaveAddressIIC, 0x28, 0x11);




          /* Enable FIFO with a Watermark of 10 */
       IIC_RegWrite(SlaveAddressIIC, 0x0F, 0x4A);                                  No           Check for interrupt
                                                                                                  on pin INT2




                                                                                                           Yes


                                                                                             /* Read INT_SOURCE */
                                                                                  INT_S = IIC_RegRead(SlaveAddressIIC, 0x12);




                                                              Service interrupt
                                                                 and clear


                                                                                  No          /* Is SRC_FIFO Set */
                                                                                            (INT_S & 0x40) == TRUE




                                                                                     /* Read FIFO status, this clears the INT */
                                                                                   F_S = IIC _RegRead(SlaveAddressIIC, 0x0D);




                                                                       No                   /* Is F_WMRK_FLAG Set */
                                                                                               (F_S & 0x40) == TRUE




                                                                                                           Yes


                                                                    /* Dump FIFO with a multi byte I2C read */
                                                     IIC _RegReadN(SlaveAddressIIC, 0x01, 32, &fifo_data[0].Sample.BT.b_msb);


                                                           Figure 6. FIFO Setup




                                                                                                                                   MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                             11
5        Terminology
5.1      Resolution
    The resolution of a pressure sensor is the minimum change of pressure that can be reliably measured. The usable resolution
of the MPL3115A2 is programmable, enabling the user to choose a compromise between acquisition speed, power consumption,
and resolution that best fits the application. To simplify the programming, the data is always reported in the same format with
differing number of usable bits.

5.2      Accuracy
5.2.1    Offset
    The offset is defined as the output signal obtained when the Reference Pressure (a vacuum for an absolute pressure sensor)
is applied to the sensor. Offset error affects absolute pressure measurements but not relative pressure measurements. An alti-
tude measurement is the pressure value in comparison to sea level, a Barometric measurement is the pressure value read by
the sensor, i.e. a measurement of total pressure seen (e.g. 70 kPa), or total height (e.g. 3000 m) above sea level. A change in
the offset will affect the pressure value or height seen above sea level as it shifts the sea level ‘base reference’. An absolute
pressure measurement is not the same as relative pressure measurement, where the pressure is compared when raising or low-
ering pressure in shorter intervals. This would be a walk up a hill, measuring the pressure and altitude difference from start to
finish. In the relative case, the offset shifts are shared in the two absolute measurements and negate each other during the pres-
sure calculation. For the MPL3115A2, the long term offset shift can be removed by adjusting the pressure or altitude offset cor-
rection. This adjustment is provided to override the factory programmed values to compensate for offsets introduced by
manufacturing and mounting stresses. It is highly recommended that this is utilized to realize the full accuracy potential of the
device.

5.2.2    Linearity
  Linearity compares the slope of the measurement data to that of an ideal transfer function. It refers to how well a transducer’s
output follows the equation Pout = Poff + Sensitivity x P straight line equation over the operating pressure range. The method
used by Freescale to give the linearity spec is the end-point straight line method measured at midrange pressure.

5.2.3    Absolute Pressure
    Absolute pressure sensors measure an external pressure relative to a zero-pressure reference (vacuum) sealed inside the
reference chamber of the die during manufacturing. This standard allows comparison to a standard value set such that
14.7 psi = 101325 Pa = 1 atm at sea level as a measurement target. The absolute pressure is used to determine altitude as it
has a constant reference for comparison. Measurement at sea level can be compared to measurement at a mountain summit as
they use the same vacuum reference. The conversion of absolute pressure to altitude in meters is calculated based on US Stan-
dard Atmosphere 1976 (NASA). Note that absolute pressure is not linear in nature to altitude; it is an exponential function. The
value of altitude, in increments of 0.0625 meters, can be read directly from the MPL3115A2, or the value of pressure in
0.25 Pascal (Pa) units.

5.2.4    Span
   Span is the value of full scale output with offset subtracted, representing the full range of the pressure sensor. Ideally the span
is a specification over a constant temperature. The MPL3115A2 uses internal temperature compensation to remove drift. Span
accuracy is the comparison of the measured difference and the actual difference between the highest and lowest pressures in
the specified range.

5.3      Pressure/Altitude
    The MPL3115A2 device is a high accuracy pressure sensor with integrated data calculation and logging capabilities. The al-
titude calculations are based on the measured pressure, the user input of the equivalent sea level pressure (to compensate for
local weather conditions) and the US Standard Atmosphere 1976 (NASA) to give the altitude readings. Pressure is given in Pas-
cals (Pa), and fractions of a Pa. Altitude is given in meters (m) and fractions of a meter. The altitude is calculated from the pres-
sure using the equation below:

                                                             0.1902632
                       h = 44330.77 { 1 – ( p ⁄ p 0 )                    } + OFF_H (Register Value)
  Where p0 = sea level pressure (101326 Pa) and h is in meters. The MPL3115A2 uses this value since the offset register is
defined as 2 Pascals per LSB.




MPL3115A2

                                                                                                                        Sensors
12                                                                                                   Freescale Semiconductor, Inc.
5.4        Power Modes of Operation




                                                                                     SLEEP

                                                                                                              ACTIVE




                     OFF                            STANDBY                                                    WAKE



                                                 Figure 7. Mode Transition Diagram

5.4.1      Off
    Unit is powered down and has no operating functionality. VDD and VDDIO are not powered.

5.4.2      STANDBY
  The digital sections are operational and the unit is capable of receiving commands and delivering stored data. The analog
sections are off. The part is waiting for CTRL_REG1 to be configured and the part to enter active mode.

5.4.3      ACTIVE
  Both analog and digital sections are running. The unit is capable of gathering new data, and accepting commands.
MPL3115A2 is fully functional.

Table 7. Mode of Operation Description
    Mode                  I2C Bus State                   VDD                                             Function Description

    OFF      Powered Down                                < 1.62 V         < VDD + 0.3 V         Device is powered off.

                                                                    SBYB bit of CTRL_REG1 is Only POR and digital blocks are enabled.
STANDBY I2C Communication with device is possible          ON
                                                                             cleared         Analog subsystem is disabled.

                                                                    SBYB bit of CTRL_REG1 is
 ACTIVE      I2C Communication with device is possible     ON
                                                                                set
                                                                                             All blocks are enabled (POR, digital, analog).



6          Functionality
   The device is a low-power, high accuracy digital output altimeter, barometer and thermometer, packaged in a 3 x 5 x 1.1 mm
form factor. The complete device includes a sensing element, analog and digital signal processing and an I2C interface.
   The device has two operational modes, barometer and altimeter. Both modes include a thermometer temperature output func-
tion.
   Power consumption and sensitivity are programmable where the data oversampling ratio can be set to balance current con-
sumption and noise/resolution. Serial Interface Communications is through an I2C interface thus making the device particularly
suitable for direct interfacing with a microcontroller. The device features two independently programmable interrupt signals INT1
and INT2. These can be set to generate an interrupt signal when a new set of Pressure/Altitude and Temperature data is avail-
able, thereby simplifying data acquisition for the host controller. These interrupt pins can also be configured to generate interrupts
when a user programmed set of conditions are met (see Section 6.6, “External Interrupts”). Examples are: Interrupt can be trig-
gered when a single new data acquisition is ready, when a desired number of samples are stored within the internal FIFO or when
a change of Pressure/Altitude or Temperature is detected.
    In RAW mode, the FIFO must be disabled and all other functionality: Alarms, Deltas and other interrupts are disabled.




                                                                                                                             MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                           13
6.1        Factory Calibration
   The device is factory calibrated for sensitivity, offset for both Temperature and Pressure measurements. Trim values are
stored, on-chip, in Non-Volatile Memory (NVM). In normal use, further calibration is not necessary; however, in order to realize
the highest possible accuracy, the device allows the user to override the factory set offset values after power-up. The user ad-
justments are stored in volatile registers. The factory calibration values are not affected, and are always used by default on power-
up.

6.2        Barometer/Altimeter Function
   The mode of operation of the device can be selected as Barometer or Altimeter. The internal sensor gives an absolute pressure
signal. The absolute pressure signal is processed to provide a scaled pressure or an altitude, depending on the mode selected.
The combination of a high performance sensor and the signal processing enable resolution of pressures below 1 Pa and altitude
resolution of better than 1 m at sea level.
    When in Barometer mode, all pressure related data is reported as 20-bit unsigned data in Pascals. When in Altimeter mode,
all pressure data is converted to equivalent altitude, based on the US standard atmosphere and then stored as 20-bit 2’s com-
plement value in meters and fractions of a meter.

6.2.1      Barometric Input
  In order to accurately determine the altitude by pressure, a facility is provided to input the local barometric pressure correction.
The default value is 101,326 Pa since the register is in units of 2 Pascals per LSB.

6.3        Temperature Function
   The unit contains a high resolution temperature sensor that provides data to the user via a 16-bit data register, as well as for
internal compensation of the pressure sensor.

6.4        Autonomous Data Acquisition
  The unit can be programmed to periodically capture Altitude/Pressure and Temperature data. Up to 32 data acquisitions can
be stored in the internal FIFO. The interval between acquisitions is programmable from 1 second to 9 hours.
  Data: (Up to 32 samples over 12 days). The unit can also be programmed to make a single reading and then go to standby
mode.

6.5         FIFO
   A 32-sample FIFO is incorporated to minimize the overhead of collecting multiple data samples. The FIFO stores both Tem-
perature and Pressure/Altitude data. The device can be programmed to autonomously collect data at programmed intervals and
store the data in the FIFO. FIFO interrupts can be triggered by watermark full or data contention (FIFO GATE) events.

6.6        External Interrupts
  Two independent interrupt out pins are provided. The configuration of the pins are programmable (polarity, open drain or push-
pull.) Any one of the internal interrupt sources can be routed to either pin.

6.6.1      Reach Target Threshold Pressure/Altitude (SRC_PTH)
   The interrupt flag is set on reaching the value stored in the Pressure/Altitude target register. Additionally, a window value pro-
vides the ability to signal when the target is nearing from either above or below the value in the Pressure/Altitude target register.
When in barometer mode, these values represent pressures rather than altitudes.
     Examples:
        • Set Altitude alert to 3000m and window value to 100m, interrupt is asserted passing 2900m, 3000m, and 3100m.
        • Set Pressure alert to 100.0 kPa and window value to 5 kPa, interrupt can be sent passing 95 kPa, 100 kPa and 105 kPa.
     Note: When the window value is set to 0 then the interrupt will only be generated when reaching or crossing the target value.

6.6.2      Reach Window Target Pressure/Altitude (SRC_PW)
     The interrupt flag is set when the pressure/altitude value is within the window defined by the following formula:

                                   Window = P_TGT MSB ,LSB ± P_WND MSB ,LSB
     Note: No interrupt is generated if the P_WND value is set to 0.



MPL3115A2

                                                                                                                        Sensors
14                                                                                                   Freescale Semiconductor, Inc.
6.6.3    Reach Target Threshold Temperature (SRC_TTH)
   Interrupt flag is set on reaching the value stored in the Temperature target register. Additionally a window value provides ability
to signal when the target is nearing from either above or below the value in the Temperature target register.
  Note: When the window value is set to 0 then the interrupt will only be generated when reaching or crossing the target value.

6.6.4    Reach Window Target Temperature (SRC_TW)
  The interrupt flag is set when the temperature value is within the window defined by the following formula:


                                             Window = T_TGT ± T_WND
  Note: No interrupt is generated if the T_WND value is set to 0.

6.6.5    Pressure/Altitude Change (SRC_PCHG)
  Interrupt flag is set if sequential Pressure/Altitude acquisitions exceed value stored in Pressure/Altitude window value register.

6.6.6    Temperature Change (SRC_TCHG)
  Interrupt flag is set if sequential Temperature acquisitions exceed the value stored in Pressure/Altitude window value register.

6.6.7    Data Ready
  Interrupt flag is set when new data or a data overwrite event has occurred. PTOW and/or PTDR (DR_STATUS register) must
be set for an interrupt to be generated.

6.6.8    FIFO Event
   Interrupt flag is set when either an overflow or watermark event has occurred. For more information please see Section 7.3,
“FIFO Setup”.

6.7      Pressure/Altitude and Temperature Delta
  Registers show the differences from the last Pressure/Altitude and Temperature samples.

6.8      Min/Max Data Value Storage
  Registers record the minimum and maximum Pressure/Altitude and Temperature.

6.9      Digital Interface
  The registers embedded inside the device are accessed through an I2C serial interface.

                                      Table 8. Serial Interface Pin Descriptions
                                              Name                      Description

                                               SCL          I2C Serial Clock

                                              SDA           I2C Serial Data

   There are two signals associated with the I2C bus: the Serial Clock Line (SCL) and the Serial Data line (SDA). The latter is a
bidirectional line used for sending and receiving the data to/from the interface. External pull-up resistors connected to VDD are
expected for SDA and SCL. When the bus is free both the lines are high. The I2C interface is compliant with fast mode (400 kHz),
and normal mode (100 kHz) I2C standards

6.9.1    I2C Operation
   The transaction on the bus is started through a start condition (START) signal. START condition is defined as a HIGH to LOW
transition on the data line while the SCL line is held HIGH. After START has been transmitted by the master, the bus is considered
busy. The next byte of data transmitted after START contains the slave address in the first 7 bits, and the eighth bit tells whether
the master is receiving data from the slave or transmitting data to the slave. When an address is sent, each device in the system
compares the first seven bits after a start condition with its address. If they match, the device considers itself addressed by the
master. The ninth clock pulse, following the slave address byte (and each subsequent byte) is the acknowledge (ACK). The trans-
mitter must release the SDA line during the ACK period. The receiver must then pull the data line low so that it remains stable
low during the high period of the acknowledge clock period.



                                                                                                                        MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                      15
   The number of bytes per transfer is unlimited. If the master can't receive another complete byte of data until it has performed
some other function, it can hold the clock line, SCL low to force the transmitter into a wait state. Data transfer only continues when
the master is ready for another byte and releases the clock line.
   A low to high transition on the SDA line while the SCL line is high is defined as a stop condition (STOP). A data transfer is
always terminated by a STOP. A master may also issue a repeated START during a data transfer. Device expects repeated
STARTs to be used to randomly read from specific registers.
     The standard 7-bit I2C slave address is 0x60 or 1100000. 8-bit read is 0xC1, 8-bit write is 0xC0.
     Consult factory for alternate addresses. See the application note titled Sensor I2C Setup and FAQ (document AN4481).




MPL3115A2

                                                                                                                        Sensors
16                                                                                                   Freescale Semiconductor, Inc.
7        Register Descriptions
Table 9. Register Address Map
                                                        Reset
                                                        when
    Register                                                            Auto-Increment
                            Name                  Reset STBY     Type                                         Comment
    Address                                                                Address
                                                          to
                                                        Active
                    Sensor Status Register
      0x00                                        0x00   Yes      R            0x01             Alias for DR_STATUS or F_STATUS
                       (STATUS)(1)(2)
                                                                                                                       Root pointer to
                                                                                              Bits 12-19 of 20-bit
                    Pressure Data Out MSB                                                                               Pressure and
      0x01                                        0x00   Yes      R     0x02          0x01    real-time Pressure
                      (OUT P_MSB)(1)(2)                                                                               Temperature FIFO
                                                                                                    sample.
                                                                                                                            data.
                    Pressure Data Out CSB
      0x02                                        0x00   Yes      R            0x03          Bits 4-11 of 20-bit real-time Pressure sample
                      (OUT_P_CSB)(1)(2)
                    Pressure Data Out LSB
      0x03                                        0x00   Yes      R            0x04          Bits 0-3 of 20-bit real-time Pressure sample
                     (OUT_P _LSB)(1)(2)
                  Temperature Data Out MSB                                                    Bits 4-11 of 12-bit real-time Temperature
      0x04                                        0x00   Yes      R            0x05
                     (OUT_T_MSB)(1)(2)                                                                         sample
                  Temperature Data Out LSB                                                     Bits 0-3 of 12-bit real-time Temperature
      0x05                                        0x00   Yes      R            0x00
                     (OUT_T _LSB)(1)(2)                                                                         sample
                    Sensor Status Register
    0x06/0x00                                     0x00   Yes      R            0x07                Data Ready status information
                     (DR_STATUS)(1)(2)
                 Pressure Data Out Delta MSB
      0x07                                        0x00   Yes      R            0x08           Bits 12-19 of 20-bit Pressure change data
                  (OUT_P_DELTA MSB)(1)(2)
                 Pressure Data Out Delta CSB
      0x08                                        0x00   Yes      R            0x09           Bits 4-11 of 20-bit Pressure change data
                  (OUT_P_DELTA_CSB)(1)(2)
                 Pressure Data Out Delta LSB
      0x09                                        0x00   Yes      R         0x0A               Bits 0-3 of 20-bit Pressure change data
                  (OUT_P_DELTA_LSB)(1)(2)
                Temperature Data Out Delta MSB
      0x0A                                        0x00   Yes      R         0x0B             Bits 4-11 of 12-bit Temperature change data
                  (OUT_T_DELTA_MSB)(1)(2)
                Temperature Data Out Delta LSB
      0x0B                                        0x00   Yes      R            0x06          Bits 0-3 of 12-bit Temperature change data
                  (OUT_T_DELTA_LSB)(1)(2)
                 Device Identification Register
      0x0C                                        0xC4    No      R         0x0D                       Fixed Device ID Number
                        (WHO_AM_I)
                     FIFO Status Register
      0x0D                                        0x00   Yes      R         0x0E                FIFO Status: No FIFO event detected
                       (F_STATUS)(1)(2)
                    FIFO 8-bit Data Access
    0X0E/0x01                                     0x00   Yes      R         0x0E                       FIFO 8-bit data access
                        (F_DATA)(1)(2)
                     FIFO Setup Register
      0x0F                                        0x00    No     R/W           0x10                           FIFO setup
                       (F_SETUP)(1)(3)
                          Time Delay
      0x10                                        0x00   Yes      R            0x11                   Time since FIFO overflow
                   Register(TIME_DLY)(1)(2)
                    System Mode Register
      0x11                                        0x00   Yes      R            0x12                     Current system mode
                        (SYSMOD)(2)
                   Interrupt Source Register
      0x12                                        0x00    No      R            0x13                         Interrupt status
                       (INT_SOURCE)(1)
                PT Data Configuration Register
      0x13                                        0x00    No     R/W           0x14                 Data event flag configuration
                    (PT_DATA_CFG)(1)(3)
                      BAR Input in MSB                                                         Barometric input for Altitude calculation
      0x14                                        0xC5    No     R/W           0x15
                     (BAR_IN_MSB)(1)(3)                                                                       bits 8-15
                       BAR Input in LSB                                                      Barometric input for Altitude calculation bits
      0x15                                        0xE7    No     R/W           0x16
                      (BAR_IN_LSB)(1)(3)                                                                          0-7




                                                                                                                               MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                           17
Table 9. Register Address Map
                       Pressure Target MSB
     0x16                                             0x00   No     R/W           0x17           Pressure/Altitude target value bits 8-15
                        (P_TGT_MSB)(1)(3)
                       Pressure Target LSB
     0x17                                             0x00   No     R/W           0x18            Pressure/Altitude target value bits 0-7
                        (P_TGT_LSB)(1)(3)
     0x18        Temperature Target (T_TGT)(1)(3)     0x00   No     R/W           0x19                  Temperature target value
                  Pressure/Altitude Window MSB                                                       Pressure/Altitude window value
     0x19                                             0x00   No     R/W           0x1A
                       (P_WND_MSB)(1)(3)                                                                       bits 8-15
                  Pressure/Altitude Window LSB                                                       Pressure/Altitude window value
     0x1A                                             0x00   No     R/W           0X1B
                       (P_WND_LSB)(1)(3)                                                                         bits 0-7
                       Temperature Window
     0X1B                                             0x00   No     R/W           0x1C                 Temperature window value
                          (T_WND)(1)(3)
                 Minimum Pressure Data Out MSB
     0x1C                                             0x00   No     R/W           0x1D            Minimum Pressure/Altitude bits 12-19
                       (P_MIN_MSB)(1)(3)
                 Minimum Pressure Data Out CSB
     0x1D                                             0x00   No     R/W           0x1E            Minimum Pressure/Altitude bits 4-11
                       (P_MIN_CSB)(1)(3)
                 Minimum Pressure Data Out LSB
     0x1E                                             0x00   No     R/W           0x1F             Minimum Pressure/Altitude bits 0-3
                       (P_MIN_LSB)(1)(3)
                 Minimum Temperature Data Out
     0x1F                                             0x00   No     R/W           0x20              Minimum Temperature bits 8-15
                     MSB (T_MIN_MSB)(1)(3)
                 Minimum Temperature Data Out
     0x20                                             0x00   No     R/W           0x21               Minimum Temperature bits 0-7
                     LSB (T_MIN_LSB)(1)(3)
                   Maximum Pressure Data Out
     0x21                                             0x00   No     R/W           0x22           Maximum Pressure/Altitude bits 12-19
                    MSB (P_MAX_MSB)(1)(3)
                   Maximum Pressure Data Out
     0x22                                             0x00   No     R/W           0x23            Maximum Pressure/Altitude bits 4-11
                    CSB (P_MAX_CSB)(1)(3)
                 Maximum Pressure Data Out LSB
     0x23                                             0x00   No     R/W           0X24             Maximum Pressure/Altitude bits 0-3
                      (P_MAX_LSB)(1)(3)
                 Maximum Temperature Data Out
     0X24                                             0x00   No     R/W           0x25              Maximum Temperature bits 8-15
                    MSB (T_MAX_MSB)(1)(3)
                 Maximum Temperature Data Out
     0x25                                             0x00   No     R/W           0x26               Maximum Temperature bits 0-7
                    LSB (T_MAX_LSB)(1)(3)
                         Control Register 1
     0x26                                             0x00   No     R/W           0x27                    Modes, Oversampling
                        (CTRL_REG1)(1)(4)
                         Control Register 2
     0x27                                             0x00   No     R/W           0x28                    Acquisition time step
                        (CTRL_REG2)(1)(4)
                         Control Register 3
     0x28                                             0x00   No     R/W           0x29                  Interrupt pin configuration
                        (CTRL_REG3)(1)(4)
                         Control Register 4
     0x29                                             0x00   No     R/W           0x2A                      Interrupt enables
                        (CTRL_REG4)(1)(4)
                         Control Register 5
     0x2A                                             0x00   No     R/W           0x2B               Interrupt output pin assignment
                        (CTRL_REG5)(1)(4)
                    Pressure Data User Offset
     0x2B                                             0x00   No     R/W           0x2C                     Pressure data offset
                        Register (OFF_P)
                  Temperature Data User Offset
     0x2C                                             0x00   No     R/W           0x2D                   Temperature data offset
                      Register (OFF_T)
                 Altitude Data User Offset Register
     0x2D                                             0x00   No     R/W           0x0C                     Altitude data offset
                              (OFF_H)

1. Register contents are preserved when transitioning from “ACTIVE” to “STANDBY” mode.
2. Register contents are reset when transitioning from “STANDBY” to “ACTIVE” mode.
3. Register contents can be modified anytime in “STANDBY” or “ACTIVE” mode.
4. Modification of this register’s contents can only occur when device in “STANDBY” mode except the SBYB, OST and RST bit fields in
   CTRL_REG1 register.




MPL3115A2

                                                                                                                           Sensors
18                                                                                                      Freescale Semiconductor, Inc.
Table 10. Register Address Map with FIFO Modes(1)
                                                              Reset
 Register                                           Reset   when             Auto-Increment
                            Name                                  Type                                         Comment
 Address                                            Value STANDBY               Address
                                                            to ACTIVE

                                             Registers: Area A (F_Mode = 0, FIFO disabled)
  0x00/      Sensor Status Register (DR_STATUS)      0x00     Yes       R          0x01                       DR_STATUS
  0x06
                                                                                                                         Root pointer to
                                                                                                 Bits 12-19 of 20-bit
                                                                                                                         Pressure and
   0x01      Pressure Data Out MSB (OUT P_MSB)       0x00     Yes       R      0x02    0x01      real-time Pressure
                                                                                                                          Temperature
                                                                                                       sample.
                                                                                                                           FIFO data.

                                                                                                 Bits 4-11 of 20-bit real-time Pressure
   0x02      Pressure Data Out CSB (OUT_P_CSB)       0x00     Yes       R          0x03
                                                                                                                sample

                                                                                                  Bits 0-3 of 20-bit real-time Pressure
   0x03      Pressure Data Out LSB (OUT_P _LSB)      0x00     Yes       R          0x04
                                                                                                                 sample

                  Temperature Data Out MSB                                                      Bits 4-11 of 12-bit real-time Temperature
   0x04                                              0x00     Yes       R          0x05
                       (OUT_T_MSB)                                                                               sample

                  Temperature Data Out LSB                                                      Bits 0-3 of 12-bit real-time Temperature
   0x05                                              0x00     Yes       R          0x00
                      (OUT_T _LSB)                                                                               sample

                               Registers: Area A (F_Mode > 0, FIFO in circular buffer or full stop mode)
  0x00/
              Sensor Status Register (F_STATUS)      0x00     Yes       R          0x01                        F_STATUS
  0x0D

   0x01        FIFO 8-bit Data Access (F_DATA)       0x00     Yes       R          0x01                           0x01

   0x02        Read to Reserved Area returns 00      0x00     N/A       —          0x03                             —

   0x03        Read to Reserved Area returns 00      0x00     N/A       —          0x04                             —

   0x04        Read to Reserved Area returns 00      0x00     N/A       —          0x05                             —

   0x05        Read to Reserved Area returns 00      0x00     N/A       —          0x00                             —

1. The Registers in Area A from 0x00 to 0x05 depend on the F_MODE bit setting in FIFO Setup Register (F_SETUP).
    • F_MODE = 00, FIFO is disabled.
    • F_MODE = 01 is circular buffer.
    • F_MODE = 10 is full stop mode.


7.1         Sensor Status
7.1.1       Status (0x00)
                 Table 11. Alias for DR_Status (0x06) or F_Status (0x0D)
                        FIFO Data Enabled Mode Bit Setting
                                   F_MODE = 00(1)                              0x00 = DR_STATUS (0x06)

                                   F_MODE > 00                                  0x00 = F_STATUS (0x0D)

                 1. The F_MODE is defined in Section 7.3.3, “F_SETUP (0x0F)”

  The aliases allow the STATUS register to be read easily before reading the current Pressure/Altitude or Temperature data,
delta Pressure/Altitude or Temperature data or FIFO data using register address auto-incrementing mechanism.

7.1.2       DR_STATUS (0x06)
  The STATUS register provides the acquisition status information on a per sample basis, and reflects real-time updates to the
OUT_P and OUT_T registers. The same STATUS register can be read through an alternate address.




                                                                                                                          MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                         19
7.1.2.1         Data Registers with F_MODE = 00 (FIFO Disabled)
   When the FIFO subsystem data output register driver is disabled (F_MODE = 00), the register indicates the real-time status
information of the sample data.

Table 12. DR_STATUS Register
                    7               6               5                4               3               2                1               0
            R     PTOW            POW             TOW                0            PTDR             PDR              TDR               0
            W
       Reset        0               0               0                0               0               0                0               0


Table 13. DR_STATUS Bit Descriptions
     Name                                                                Description
                  Pressure/Altitude OR Temperature data overwrite. PTOW is set to 1 whenever new data is acquired before completing the
                  retrieval of the previous set. This event occurs when the content of at least one data register (i.e. OUT_P, OUT_T) has been
                  overwritten. PTOW is cleared when the high-bytes of the data (OUT_P_MSB or OUT_T_MSB) are read, when F_MODE is
                  zero. PTOW is cleared by reading F_DATA register when F_MODE > 0.
     PTOW
                  Default value: 0
                  0: No data overwrite has occurred
                  1: Previous Pressure/Altitude or Temperature data was overwritten by new Pressure/Altitude or Temperature data before it
                  was read

                  Pressure/Altitude data overwrite. POW is set to 1 whenever a new Pressure/Altitude acquisition is completed before the
                  retrieval of the previous data. When this occurs the previous data is overwritten. POW is cleared anytime OUT_P_MSB
                  register is read, when F_MODE is zero. POW is cleared by reading F_DATA register when F_MODE > 0.
      POW
                  Default value: 0
                  0: No data overwrite has occurred
                  1: Previous Pressure/Altitude data was overwritten by new Pressure/Altitude data before it was read

                  Temperature data overwrite. TOW is set to 1 whenever a new Temperature acquisition is completed before the retrieval of
                  the previous data. When this occurs the previous data is overwritten. TOW is cleared anytime OUT_T_MSB register is read,
                  when F_MODE is zero. TOW is cleared by reading F_DATA register when F_MODE > 0.
      TOW
                  Default value: 0
                  0: No data overwrite has occurred
                  1: Previous Temperature data was overwritten by new Temperature data before it was read

                  Pressure/Altitude OR Temperature data ready. PTDR signals that a new acquisition for either Pressure/Altitude or
                  Temperature is available. PTDR is cleared anytime OUT_P_MSB or OUT_T_MSB register is read, when F_MODE is zero.
                  PTDR is cleared by reading F_DATA register when F_MODE > 0.
     PTDR
                  Default value: 0
                  0: No new set of data ready
                  1: A new set of data is ready

                  Pressure/Altitude new data available. PDR is set to 1 whenever a new Pressure/Altitude data acquisition is completed. PDR
                  is cleared anytime OUT_P_MSB register is read, when F_MODE is zero. PDR is cleared by reading F_DATA register when
      PDR         F_MODE > 0.
                  0: No new Pressure/Altitude data is available
                  1: A new set of Pressure/Altitude data is ready

                  Temperature new Data Available. TDR is set to 1 whenever a Temperature data acquisition is completed. TDR is cleared
                  anytime OUT_T_MSB register is read, when F_MODE is zero. TDR is cleared by reading F_DATA register when F_MODE
      TDR         > 0.
                  0: No new Temperature data ready
                  1: A new Temperature data is ready

  PDR and POW flag generation is required for the Pressure/Altitude event flag generator to be enabled (PDEFE = 1) in the PT
Data Configuration Register (PT_DATA_CFG).
  TDR and TOW flag generation is required for the Temperature event flag generator to be enabled (TDEFE = 1) in the PT Data
Configuration Register (PT_DATA_CFG).
  PTDR and PTOW flag generation requires the DREM event flag generator to be enabled in the PT Data Configuration Register
(PT_DATA_CFG).




MPL3115A2

                                                                                                                              Sensors
20                                                                                                         Freescale Semiconductor, Inc.
7.1.3      OUT_P_MSB (0x01), OUT_P_CSB (0x02), OUT_P_LSB (0x03), OUT_T_MSB (0x04),
           OUT_T_LSB (0x05)
   The Altitude data is stored as a signed fractional 20-bit value in meters in Q16.4 format. The OUT_P_MSB and OUT_P_CSB
registers contain the integer part in meters and the OUT_P_LSB register contains the fractional part. Left shifting the
OUT_T_MSB byte by 24 bits into a 32 variable and doing a logical OR with the OUT_T_CSB byte left shifted 16 bits and a logical
OR with the OUT_T_LSB byte left shifted 8 bits gives the altitude in meters times 65536.
   The Pressure data is stored as an unsigned fractional 20-bit value in Pascals in Q18.2 format. The OUT_P_MSB and
OUT_P_CSB registers and bits 7-6 of the OUT_P_LSB register contain the integer part in Pascals. Bits 5-4 of OUT_P_LSB con-
tain the fractional component. Left shifting the OUT_T_MSB byte by 16 bits into a 32 variable and doing a logical OR with the
OUT_T_CSB byte left shifted 8 bits and a logical OR with the OUT_T_LSB byte gives the pressure in Pascals times 64.
  When RAW bit is set (CTRL_REG1), then the RAW value is stored in all 24 bits of OUT_P_MSB, OUT_P_CSB and
OUT_P_LSB.
   The Temperature data is stored as a signed fractional 12-bit value in °C in Q12.4 format. The OUT_T_MSB register contains
the integer part in °C and the OUT_T_LSB register contains the fractional part. Left shifting the OUT_T_MSB byte by 8 bits into
a 16-bit variable and adding the OUT_T_LSB byte with a logical OR gives the temperature in °C times 256. When RAW is se-
lected then the RAW value is stored in all 16 bits of OUT_T_MSB and OUT_T_LSB.
7.1.3.1           Data Registers with F_MODE = 00
   The DR_STATUS register, OUT_P_MSB, OUT_P_CSB, OUT_P_LSB, OUT_T_MSB, and OUT_T_LSB are stored in the auto-
incrementing address range of 0x00 to 0x05. This allows the host controller to read the status register followed by the
20-bit Pressure/Altitude and 12-bit Temperature in a 6-byte, I2C transaction.

Table 14. OUT_P_MSB Register
                      7            6              5             4              3             2              1             0
             R       PD19        PD18           PD17          PD16           PD15          PD14           PD13          PD12
             W
          Reset       0            0              0             0              0             0              0             0


Table 15. OUT_P_CSB Register
                      7            6              5             4              3             2              1             0
             R       PD11        PD10           PD9            PD8           PD7            PD6           PD5            PD4
             W
          Reset       0            0              0             0              0             0              0             0


Table 16. OUT_P_LSB Register
                      7            6              5             4              3             2              1             0
             R       PD3          PD2           PD1            PD0             0             0              0             0
             W
          Reset       0            0              0             0              0             0              0             0


Table 17. OUT_T_MSB Register
                      7            6              5             4              3             2              1             0
             R       TD11        TD10           TD9            TD8           TD7            TD6           TD5            TD4
             W
          Reset       0            0              0             0              0             0              0             0


Table 18. OUT_T_LSB Register
                      7            6              5             4              3             2              1             0
             R       TD3          TD2           TD1            TD0             0             0              0             0
             W
          Reset       0            0              0             0              0             0              0             0

  If the FIFO data output register driver is enabled (F_MODE > 00), register 0x01 points to the FIFO read pointer, while registers
0x02, 0x03, 0x04, 0x05, return a value of zero when read.


                                                                                                                    MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                  21
7.1.4      OUT_P_DELTA_MSB (0x07), OUT_P_DELTA_CSB (0x08), OUT_P_DELTA_LSB (0x09),
           OUT_T_ DELTA_MSB (0x0A), OUT_T_DELTA_LSB (0x0B)
     Differences from last Pressure/Altitude and Temperature samples.
  The Altitude data is arranged as 20-bit 2’s complement value in meters. Stored as meters with the 16 bits of
OUT_P_DELTA_MSB and OUT_P_DELTA_CSB and with fractions of a meter stored in 4 bits in position 7-4 of
OUT_P_DELTA_LSB.
   The Pressure is arranged as 20-bit 2’s complement value in Pascals. The first 18 bits are located in OUT_P_DELTA_MSB,
OUT_P_DELTA_CSB and bits 7-6 of OUT_P_DELTA_LSB. The 2 bits in position 5-4 of OUT_P_DELTA_LSB represent the frac-
tional component.
  The Temperature data is arranged as 12-bit values in degrees C. The 8 bits of OUT_T_DELTA_MSB representing degrees
and with fractions of a degree stored in 4 bits in position 7-4 of OUT_T_DELTA_LSB.
     In RAW mode, these registers are not used and their values are not updated.
  Note: The OUT_P_DELTA, OUT_T_DELTA registers store the difference data information regardless of the state of the FIFO
data output register driver bit, F_MODE > 00.

Table 19. OUT_P_DELTA _MSB Register
                      7             6            5             4             3            2              1            0
             R     PDD19         PDD18         PDD17        PDD16         PDD15         PDD14         PDD13         PDD12
             W
          Reset       0             0            0             0             0            0              0            0


Table 20. OUT_P_DELTA_CSB Register
                      7            6             5             4             3            2              1            0
             R     PDD11         PDD10         PDD9          PDD8         PDD7          PDD6           PDD5         PDD4
             W
          Reset       0            0             0             0             0            0              0            0


Table 21. OUT_P_DELTA_LSB Register
                      7             6            5             4             3             2             1            0
             R      TDD3          TDD2         TDD1          TDD0            0             0             0            0
             W
          Reset       0             0            0             0             0             0             0            0


Table 22. OUT_T_DELTA _MSB Register
                      7             6            5             4             3            2              1            0
             R     TDD11         TDD10         TDD9          TDD8          TDD7         TDD6           TDD5         TDD4
             W
          Reset       0             0            0             0             0            0              0            0


Table 23. OUT_T_DELTA_LSB Register
                      7             6            5             4             3             2             1            0
             R      TDD3          TDD2         TDD1          TDD0            0             0             0            0
             W
          Reset       0             0            0             0             0             0             0            0




MPL3115A2

                                                                                                                   Sensors
22                                                                                              Freescale Semiconductor, Inc.
7.2         Device ID
7.2.1       WHO_AM_I (0x0C)
   Device identification register. This register contains the device identifier which is set to 0xC4 by default. The value is factory
programmed. Consult factory for custom alternate values.

Table 24. WHO_AM_I Register
                        7               6             5               4                3              2             1               0
               R                                                            WHO_AM_I
               W
           Reset    (NVM data)    (NVM data)     (NVM data)      (NVM data)      (NVM data)     (NVM data)     (NVM data)      (NVM data)
                        1               1             0               0                0              1             0               0


7.3         FIFO Setup
7.3.1       F_STATUS (0x0D)
   If the FIFO subsystem data output register driver is enabled, the status register indicates the current status information of the
FIFO subsystem.

Table 25. F_STATUS
                    7               6                     5             4              3              2              1              0
           R    F_OVF        F_WMRK_FLAG            F_CNT5          F_CNT4         F_CNT3           F_CNT2       F_CNT1         F_CNT0
          W
    Reset           0               0                     0             0              0              0              0              0


Table 26. FIFO Flag Event Descriptions
  F_OVF            F_WMRK_FLAG                                                  Event Description

      0                 —           No FIFO overflow events detected.

      1                 —           FIFO overflow event detected.

      —                 0           No FIFO watermark events detected.

      —                 1           FIFO watermark event detected. FIFO sample count greater than watermark value.

   The F_OVF and F_WMRK_FLAG flags remain asserted while the event source is still active, but the user can clear the FIFO
interrupt bit flag in the interrupt source register (INT_SOURCE) by reading the F_STATUS register. Therefore the F_OVF bit flag
will remain asserted while the FIFO has overflowed and the F_WMRK_FLAG bit flag will remain asserted while the F_CNT value
is greater than then F_WMRK value.

Table 27. FIFO Sample Count Bit Descriptions
            Name                                                               Description

                            FIFO sample counter. F_CNT[5:0] bits indicate the number of samples currently stored in the FIFO buffer. Count
                            000000 indicates that the FIFO is empty.
          F_CNT[5:0]
                            Default value: 00_0000.
                            (000001 to 100000 indicates 1 to 32 samples stored in FIFO




                                                                                                                              MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                            23
7.3.2      F_DATA (0x0E)
   F_DATA is a read only address which provides access to 8-bit FIFO data. FIFO holds a maximum of 32 samples; a maximum
of 5 x 32 = 160 data bytes of samples can be read. When F_MODE bit in FIFO SETUP (F_SETUP) register is set to logic “1”,
the F_DATA pointer shares the same address location as OUT_P_MSB (0x01); therefore all accesses of the FIFO buffer data
use the I2C address 0x01. Reads from the other data registers (0x02, 0x03, 0x04, 0x05) will return a value of 0x00. Note: The
FIFO will NOT suspend to data accumulation during read transactions to F_DATA.

Table 28. F_DATA 8-bit Data Access Register
                        7                   6         5              4               3              2              1               0
              R                                                          F_DATA
              W
          Reset         0                   0         0              0               0              0              0               0

                              Table 29. Read Accesses through F_DATA
                                  1st read                          OUT_P_MSB (oldest)

                                  2nd read                          OUT_P_CSB (oldest)

                                  3rd read                          OUT_P_LSB (oldest)
                                      th read                       OUT_T_MSB (oldest)
                                  4
                                      th read
                                  5                                 OUT_T_LSB (oldest)

                                        .                                     .
                                        .                                     .
                                        .                                     .

                                                                    OUT_T_LSB (oldest)

                                                                            0x00

                                                                            0x00


7.3.3      F_SETUP (0x0F)
   A FIFO sample count exceeding the watermark event does not stop the FIFO from accepting new data.
   The FIFO update rate is dictated by the selected system acquisition rate (ST bits of CTRL_REG2).
   When a byte is read from the FIFO buffer the oldest sample data in the FIFO buffer is returned and also deleted from the front
of the FIFO buffer, while the FIFO sample count is decremented by one. It is assumed that the host application shall use the I2C
BURST read transaction to dump the FIFO.

Table 30. F_SETUP Register
                        7                   6         5              4               3              2              1               0
               R
                    F_MODE1       F_MODE0        F_WMRK5         F_WMRK4          F_WMRK3      F_WMRK2         F_WMRK1        F_WMRK0
              W
          Reset         0                   0         0              0               0              0              0               0


Table 31. F_SETUP Bit Descriptions
           Name                                                              Description
                            FIFO buffer overflow mode.
                            Default value: 0
                            00: FIFO is disabled
                            01: FIFO contains the most recent samples when overflowed (circular buffer). Oldest sample is discarded to be
                            replaced by new sample
                            10: FIFO stops accepting new samples when overflowed
     F_MODE[1:0](1)(2)(3)
                            11: Not Used
                            The FIFO is flushed whenever the FIFO is disabled, or transitioning from “STANDBY” mode to “ACTIVE” mode.
                            Disabling the FIFO (F_MODE = 00) resets the F_OVF, F_WMRK_FLAG, F_CNT to zero.
                            A FIFO overflow event (i.e. F_CNT = 32) will assert the F_OVF flag and a FIFO sample count equal to the sample
                            count watermark (i.e. F_WMRK) asserts the F_WMRK_FLAG event flag. To switch between FIFO modes, first
                            disable the FIFO and then write the new value to F_MODE.




MPL3115A2

                                                                                                                            Sensors
24                                                                                                       Freescale Semiconductor, Inc.
Table 31. F_SETUP Bit Descriptions
                             FIFO Event Sample Count Watermark.
                             Default value: 00_0000.
      F_WMRK[5:0](2)         These bits set the number of FIFO samples required to trigger a watermark interrupt. A FIFO watermark event flag
                             (F_WMRK_FLAG) is raised when FIFO sample count F_CNT[5:0] value is equal to the F_ WMRK[5:0] watermark.
                             Setting the F_WMRK[5:0] to 00_0000 will disable the FIFO watermark event flag generation.

1. This bit field can be written in ACTIVE mode.
2. This bit field can be written in STANDBY mode.
3. The FIFO mode (F_MODE) cannot be switched between the two operational modes (01 & 10).


7.4       Time Delay Register
7.4.1     TIME_DLY(0x10)
  The time delay register contains the number of ticks of data sample time since the last byte of the FIFO was written.
  This register starts to increment on FIFO overflow or data wrap and clears when last byte of FIFO is read.

Table 32. Time Delay Register
                        7               6              5               4               3              2               1              0
               R       TD7            TD6             TD5             TD4            TD3             TD2            TD1             TD0
            W
         Reset          0               0              0               0               0              0               0              0


7.5       System Mode Register
7.5.1     SYSMOD (0x11)
  The system mode register indicates the current device operating mode.

Table 33. SYSMOD Register
                        7               6              5               4               3              2               1              0
               R        0                              0               0               0              0               0          SYSMOD
            W
         Reset          0               0              0               0               0              0               0              0


Table 34. SYSMOD Bit Descriptions
        Name                                                                Description
   RESERVED          Reserved Bits 7-1, will always read 0.
                     System mode. Default value: 0.
      SYSMOD         0: STANDBY mode
                     1: ACTIVE mode


7.6       System Interrupt Status
7.6.1     INT_SOURCE (0x12)
   Interrupt source register. The bits that are set (logic ‘1’) indicate which function has asserted its interrupt and conversely, bits
that are cleared (logic ‘0’) indicate which function has not asserted its interrupt.
  The setting of the bits is rising edge sensitive, the bit is set by a low to high state change and reset by reading the appropriate
source register.

Table 35. INT_SOURCE Register
                        7               6              5               4               3              2               1              0
               R   SRC_DRDY        SRC_FIFO        SRC_PW          SRC_TW         SRC_PTH         SRC_TTH       SRC_PCHG        SRC_TCHG
            W
         Reset          0               0              0               0               0              0               0              0




                                                                                                                               MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                             25
Table 36. INT_SOURCE Bit Descriptions
         Name                                                                Description
                      Data ready interrupt status bit. Logic ‘1’ indicates that Pressure/Altitude or Temperature data ready interrupt is active
                      indicating the presence of new data and/or a data overwrite, otherwise it is a logic ‘0’.
      SRC_DRDY
                      This bit is asserted when the PTOW and/or PTDR is set and the functional block interrupt has been enabled.
                      This bit is cleared by reading the STATUS and Pressure/Temperature register.
                      FIFO interrupt status bit. Logic ‘1’ indicates that a FIFO interrupt event such as an overflow event has occurred. Logic
                      ‘0’ indicates that no FIFO interrupt event has occurred.
       SRC_FIFO
                      This bit is cleared by reading the F_STATUS register.
                      FIFO interrupt event generators: FIFO Overflow, or (Watermark: F_CNT = F_WMRK).
                      Altitude/Pressure alerter status bit near or equal to target Pressure/Altitude (near is within target value ± window value).
       SRC_PW
                      Window value needs to be non zero for interrupt to trigger.
                      Temperature alerter status bit near or equal to target temperature (near is within target value ± window value.) Window
       SRC_TW
                      value needs to be non zero for interrupt to trigger.
                      Altitude/Pressure threshold interrupt. With the window set to a non zero value, the trigger will occur on crossing any of
       SRC_PTH
                      the thresholds: upper, center or lower. If the window is set to 0, it will only trigger on crossing the center threshold.
                      Temperature threshold interrupt. With the window set to a non zero value, the trigger will occur on crossing any of the
       SRC_TTH
                      thresholds: upper, center or lower. If the window is set to 0, it will only trigger on crossing the center threshold.
      SRC_PCHG        Delta P interrupt status bit.
      SRC_TCHG        Delta T interrupt status bit




7.7       Sensor Data
7.7.1     PT_DATA_CFG (0x13)
     The PT_DATA_CFG register configures the Pressure data, Temperature data and event flag generator.

Table 37. PT_DATA_CFG Register
                     7                 6               5               4                3                2               1                0
               R     0                 0               0               0                0
                                                                                                      DREM            PDEFE            TDEFE
             W
          Reset      0                 0               0               0                0                0               0                0


Table 38. PT_DATA_CFG Bit Descriptions
        Name                                                                Description
                   Data ready event mode. If the DREM bit is set logic ‘1’ and one or more of the data ready event flags (PDEFE, TDEFE) are
                   enabled, then an event flag will be raised upon change in state of the data. If the DREM bit is cleared logic ‘0’ and one or
                   more of the data ready event flags are enabled, then an event flag will be raised whenever the system acquires a new set
       DREM        of data.
                   Default value: 0.
                   0: Event detection disabled
                   1: Generate data ready event flag on new Pressure/Altitude or Temperature data

                   Data event flag enable on new Pressure/Altitude data.
                   Default value: 0
       PDEFE
                   0: Event detection disabled
                   1: Raise event flag on new Pressure/Altitude data

                   Data event flag enable on new Temperature data. Default value: 0
       TDEFE       0: Event detection disabled
                   1: Raise event flag on new Temperature data




MPL3115A2

                                                                                                                                 Sensors
26                                                                                                            Freescale Semiconductor, Inc.
7.8      Barometric Pressure Input
7.8.1    BAR_IN_MSB, BAR_IN_LSB (0x14, 0x15)
  Barometric input for altitude calculations. Input is equivalent sea level pressure for measurement location.
  Value is input in 2 Pa units.
   Units are input as unsigned 16-bit integers. The default value is 101,326 Pa. The default value can be changed by writing to
this register

Table 39. BAR_IN_MSB Register
                     7              6            5             4             3              2             1            0
           R
                 BAR[15]          BAR[14]    BAR[13]        BAR[12]       BAR[11]        BAR[10]        BAR[9]       BAR[8]
           W
        Reset        1              1            0             0             0              1             0            1


Table 40. BAR_IN_LSB Register
                     7              6            5             4             3              2             1            0
           R
                  BAR[7]          BAR[6]      BAR[5]        BAR[4]         BAR[3]        BAR[2]         BAR[1]       BAR[0]
           W
        Reset        1              1            1             0             0              1             1            1


7.9      Pressure/Altitude Target
7.9.1    P_TGT_MSB, P_TGT_LSB (0x16, 0x17)
  Altitude/Pressure target value. This value works in conjunction with the window value (P_WND_MSB and P_WND_LSB).
  In Altitude mode, the register value is 16-bit 2’s complement value in meters.
  In Pressure mode, the value is 16-bit unsigned value in 2 Pa units.

Table 41. P_TGT_MSB Register
                     7               6           5             4              3             2              1            0
           R
                 P_TGT15          P_TGT14    P_TGT13       P_TGT12        P_TGT11       P_TGT10        P_TGT9       P_TGT8
           W
        Reset        0               0           0             0              0             0              0            0


Table 42. P_TGT_LSB Register
                     7               6           5             4              3             2              1            0
           R
                 P_TGT7           P_TGT6     P_TGT5         P_TGT4        P_TGT3         P_TGT2        P_TGT1       P_TGT0
           W
        Reset        0               0           0             0              0             0              0            0


7.10     Temperature Target
7.10.1 T_TGT (0x18)
  Temperature target value input in 2’s complement value in °C.

Table 43. T_TGT Register
                     7               6           5             4              3             2              1            0
           R
                 T_TGT7           T_TGT6     T_TGT5         T_TGT4        T_TGT3         T_TGT2        T_TGT1        T_TGT0
           W
        Reset        0               0           0             0              0             0              0            0




                                                                                                                  MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                 27
7.11       Pressure/Altitude Window
7.11.1 P_WND_MSB, P_WND_LSB (0x19, 0x1A)
  Pressure/Altitude window value. Unsigned 16-bit value of window value in meters or in 2 Pa units
Table 44. P_WND_LSB Register
                     7             6             5               4          3            2              1            0
             R
                   P_W15        P_W 14        P_W 13           P_W 12    P_W 11       P_W 10          P_W 9        P_W 8
            W
         Reset       0             0             0               0          0            0              0            0

Table 45. P_WND_MSB Description
                      7            6             5               4          3            2              1            0
             R
                    P_W7         P_W 6         P_W 5           P_W 4      P_W 3        P_W 2         P_W 1         P_W 0
             W
          Reset       0            0             0               0          0            0              0            0


7.12       Temperature Window
7.12.1 T_WND (0x1B)
     Temperature alarm window value. Unsigned 8-bit value °C.

Table 46. T_WND Register
                      7             6            5                4         3            2              1             0
             R
                    T_W7         T_W6          T_W5             T_W4      T_W3         T_W2           T_W1          T_W0
             W
          Reset       0             0            0                0         0            0              0             0


7.13       Minimum Pressure
7.13.1 P_MIN_MSB, P_MIN_CSB, P_MIN_LSB (0x1C, 0x1D, 0x1E)
     Register with captured minimum Pressure/Altitude value.
  The Altitude data is arranged as 20-bit 2’s complement value in meters. Stored as meters with the 16 bits of P_MIN_MSB and
P_MIN_CSB and with fractions of a meter stored in 4 bits in position 7-4 of P_MIN_LSB.
   The Pressure is arranged as 20-bit unsigned data in Pascals. The first 18 bits are located in P_MIN_MSB, P_MIN_CSB and
bits 7-6 of P_MIN_LSB. The 2 bits in position 5-4 of P_MIN_LSB represent the fractional component.
   The register is cleared on power-up or manually by writing ‘0’ to the register.


Table 47. P_MIN_MSB Register
                      7            6             5               4          3            2              1            0
             R
                  P_MIN 19      P_MIN 18     P_MIN 17      P_MIN 16      P_MIN 15    P_MIN 14        P_MIN 13     P_MIN 12
             W
          Reset       0            0             0               0          0            0              0            0

Table 48. P_MIN_CSB Register
                      7             6            5                4         3            2              1             0
             R
                  P_MIN 11      P_MIN 10      P_MIN 9          P_MIN 8   P_MIN 7      P_MIN 6        P_MIN 5      P_MIN 4
             W
          Reset       0             0            0                0         0            0              0             0

Table 49. P_MIN_LSB Register
                      7            6             5               4          3            2              1            0
             R                                                              0            0              0            0
                   P_MIN 3      P_MIN 2       P_MIN 1          P_MIN0
             W
          Reset       0            0             0               0          0            0              0            0




MPL3115A2

                                                                                                                  Sensors
28                                                                                             Freescale Semiconductor, Inc.
7.14    Maximum Pressure
7.14.1 P_MAX_MSB, P_MAX_CSB, P_MAX_LSB (0x21, 0x22, 0x23)
   Register with captured maximum Pressure/Altitude value.
    The Altitude data is arranged as 20-bit 2’s complement value in meters. Stored as meters with the 16 bits of P_MAX_MSB
and P_MAX_CSB and with fractions of a meter stored in 4 bits in position 7-4 of P_MAX_LSB.
   The Pressure is arranged as 20-bit unsigned data in Pascals. The first 18 bits are located in P_MAX_MSB, P_MAX_CSB and
bits 7-6 of P_MAX_LSB. The 2 bits in position 5-4 of P_MAX_LSB represent the fractional component.
   The register is cleared on power-up or manually by writing ‘0’ to the registers.
Table 50. P_MAX_MSB Register
                   7               6           5            4             3            2             1             0
           R
               P_MAX 19      P_MAX 18      P_MAX 17     P_MAX 16      P_MAX 15     P_MAX 14      P_MAX 13      P_MAX 12
          W
       Reset       0               0           0            0             0            0             0             0


Table 51. P_MAX_CSB Register
                   7               6           5            4             3            2             1             0
           R
               P_MAX 11      P_MAX 10      P_MAX 9       P_MAX 8      P_MAX 7       P_MAX 6      P_MAX 5       P_MAX 4
          W
       Reset       0               0           0            0             0            0             0             0

Table 52. P_MAX_LSB Register
                   7               6           5            4             3            2             1             0
           R                                                              0            0             0             0
                P_MAX 3         P_MAX 2    P_MAX 1       P_MAX0
          W
       Reset       0               0           0            0             0            0             0             0


7.15    Minimum Temperature
7.15.1 T_MIN_MSB, T_MIN_LSB (0x1F, 0x20)
   Register with captured minimum temperature value.
   The Temperature data is arranged as 12-bit 2’s complement value in degrees C. The 8 bits of T_MIN_MSB representing de-
grees and with fractions of a degree stored in 4 bits in position 7-4 of T_MIN_LSB.
   The register is cleared on power-up or manually by writing ‘0’ to the registers.

Table 53. T_MIN_MSB Register
                   7               6           5            4             3             2            1             0
           R
                T_MIN 11        T_MIN 10   T_MIN 9       T_MIN 8       T_MIN 7      T_MIN 6       T_MIN 5       T_MIN 4
          W
       Reset       0               0           0            0             0             0            0             0


Table 54. T_MIN_LSB Register
                   7               6           5            4             3            2             1             0
           R                                                              0            0             0             0
                T_MIN 3         T_MIN 2    T_MIN 1       T_MIN 0
          W
       Reset       0               0           0            0             0            0             0             0




                                                                                                             MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                          29
7.16        Maximum Temperature
7.16.1 T_MAX_MSB, T_MAX_LSB (0x24, 0x25)
     Register with captured maximum temperature value.
   The Temperature data is arranged as 12-bit 2’s complement value in degrees C. The 8 bits of T_MAX_MSB representing
degrees and with fractions of a degree stored in 4 bits in position 7-4 of T_MAX_LSB.
     The register is cleared on power-up or manually by writing 0 to the registers

Table 55. T_MAX_MSB Register
                         7                6               5                4                 3              2               1              0
              R
                    T_MAX 11         T_MAX 10         T_MAX 9          T_MAX 8         T_MAX 7          T_MAX 6         T_MAX 5        T_MAX 4
             W
          Reset          0                0               0                0                 0              0               0              0


Table 56. T_MAX_LSB Register
                         7                6               5                4                 3              2               1              0
              R                                                                              0              0               0              0
                     T_MAX 3         T_MAX 2          T_MAX 1         T_MAX 0
             W
          Reset          0                0               0                0                 0              0               0              0


7.17        Control Registers
   Note: Except for standby mode selection and OST (One Shot Mode), the device must be in STANDBY mode to change any
of the fields within CTRL_REG1 (0x26).

7.17.1 CTRL_REG1 (0x26)
Table 57. CTRL_REG1 Register
                         7                6               5                4                 3              2               1              0
              R                                                                                             0
                        ALT             RAW             OS2              OS1              OS0                              OST           SBYB
             W                                                                                            RST
          Reset          0                0               0                0                 0              0               0              0


Table 58. CTRL_REG1 Bit Descriptions
      Name                                                                     Description
                  This bit is sets the mode to ACTIVE, where the system will make measurements at periodic times based on the value of ST bits.
                  Default value: 0 (STANDBY)
      SBYB
                  0: Part is in STANDBY mode
                  1: Part is ACTIVE

                  OST bit will initiate a measurement immediately. If the SBYB bit is set to active, setting the OST bit will initiate an immediate
                  measurement, the part will then return to acquiring data as per the setting of the ST bits in CTRL_REG2. In this mode, the OST
                  bit does not clear itself and must be cleared and set again to initiate another immediate measurement.
      OST
                  One Shot: When SBYB is 0, the OST bit is an auto-clear bit. When OST is set, the device initiates a measurement by going into
                  active mode. Once a Pressure/Altitude and Temperature measurement is completed, it clears the OST bit and comes back to
                  STANDBY mode. User shall read the value of the OST bit before writing to this bit again.

                  Software Reset. This bit is used to activate the software reset. The Boot mechanism can be enabled in STANDBY and ACTIVE
                  mode.
                  When the Boot bit is enabled the boot mechanism resets all functional block registers and loads the respective internal registers
                  with default values.
                  If the system was already in STANDBY mode, the reboot process will immediately begin; else if the system was in ACTIVE
                  mode, the boot mechanism will automatically transition the system from ACTIVE mode to STANDBY mode, only then can the
      RST         reboot process begin.
                  The I2C communication system is reset to avoid accidental corrupted data access.
                  At the end of the boot process the RST bit is de-asserted to 0. Reading this bit will return a value of zero.
                  Default value: 0
                  0: Device reset disabled
                  1: Device reset enabled



MPL3115A2

                                                                                                                                    Sensors
30                                                                                                               Freescale Semiconductor, Inc.
Table 58. CTRL_REG1 Bit Descriptions
  OS[2:0]      Oversample Ratio. These bits select the oversampling ratio. Value is 2OS. The default value is 000 for a ratio of 1.
               RAW output mode. RAW bit will output ADC data with no post processing, except for oversampling. No scaling or offsets will be
   RAW         applied in the digital domain. The FIFO must be disabled and all other functionality: Alarms, Deltas, and other interrupts are
               disabled.
               Altimeter-Barometer mode.
               Default value: 0
    ALT
               1: Part is in Altimeter Mode
               0: Part is in Barometer mode

                      Table 59. System Output Sample Rate Selection
                                                                                                       Minimum Time
                                                                                    Oversample
                             OS2                   OS1               OS0                                  Between
                                                                                       Ratio
                                                                                                       Data Samples
                               0                    0                  0                     1                6 ms
                               0                    0                  1                     2               10 ms
                               0                    1                  0                     4               18 ms
                               0                    1                  1                     8               34 ms
                               1                    0                  0                   16                66 ms
                               1                    0                  1                   32                130 ms
                               1                    1                  0                   64                258 ms
                               1                    1                  1                   128               512 ms

  Note: RAW bit overrides the ALT mode and writes uncompensated Pressure and Temperature data

7.17.2 CTRL_REG2 (0x27)

Table 60. CTRL_REG2 Register
                 7                 6                5                  4               3               2               1               0
      R          0                 0
                                              LOAD_OUTPUT       ALARM_SEL            ST[3]           ST[2]            ST[1]           ST[0]
      W
   Reset         0                 0                0                  0               0               0               0               0


Table 61. CTRL_REG2 Bit Descriptions
      Name                                                                 Description
                     Auto acquisition time step.
     ST[3:0]         Default value: 0
                     Step value is 2ST: Giving a range of 1 second to 215 seconds (9 hours)

                     The bit selects the Target value for SRC_PW/SRC_TW and SRC_PTH/SRC_TTH
                     Default value: 0
   ALARM_SEL
                     0: The values in P_TGT_MSB, P_TGT_LSB and T_TGT are used (Default)
                     1: The values in OUT_P/OUT_T are used for calculating the interrupts SRC_PW/SRC_TW and SRC_PTH/SRC_TTH.

                     This is to load the target values for SRC_PW/SRC_TW and SRC_PTH/SRC_TTH.
                     Default value: 0
                     0: Do not load OUT_P/OUT_T as target values
 LOAD_OUTPUT         1: The next values of OUT_P/OUT_T are used to set the target values for the interrupts.
                     Note:
                     1. This bit must be set at least once if ALARM_SEL=1
                     2. To reload the next OUT_P/OUT_T as the target values clear and set again.




                                                                                                                                 MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                                 31
7.17.3 CTRL_REG3 (Interrupt CTRL Register) (0x28)

Table 62. CTRL_REG3 Register
                        7                6                 5              4                   3               2                1              0
             R          0                                                                     0
                                                         IPOL1        PP_OD1                                               IPOL2          PP_OD2
             W
         Reset          0                0                 0              0                   0               0                0              0


Table 63. CTRL_REG3 Description
         Name                                                                   Description
                        The IPOL bit selects the polarity of the interrupt signal. When IPOL is ‘0’ (default value) any interrupt event will signalled
                        with a logical ‘0'. Interrupt Polarity active high, or active low on interrupt pad INT1.
        IPOL1           Default value: 0
                        0: Active low
                        1: Active high

                        This bit configures the interrupt pin to Push-Pull or in Open Drain mode. The default value is 0 which corresponds to
                        Push-Pull mode. The open drain configuration can be used for connecting multiple interrupt signals on the same interrupt
                        line. Push-Pull/Open Drain selection on interrupt pad INT1.
       PP_OD1
                        Default value: 0
                        0: Internal Pullup
                        1: Open drain

                        Interrupt Polarity active high, or active low on interrupt pad INT2.
                        Default value: 0
        IPOL2
                        0: Active low
                        1: Active high

                        Push-Pull/Open Drain selection on interrupt pad INT2.
                        Default value: 0
       PP_OD2
                        0: Internal Pull-up
                        1: Open drain


7.17.4 CTRL_REG4 [Interrupt Enable Register] (0x29)
  The corresponding functional block interrupt enable bit allows the functional block to route its event detection flags to the sys-
tem’s interrupt controller. The interrupt controller routes the enabled functional block interrupt to the INT1 or INT2 pin.

Table 64. CTRL_REG4 Register
                    7                   6                  5              4               3               2                1                  0
         R
             INT_EN_DRDY         INT_EN _FIFO       INT_EN_PW INT_EN_TW INT_EN_PTH INT_EN_TTH INT_EN_PCHG INT_EN_TCHG
         W
     Reset          0                   0                  0              0               0               0                0                  0


Table 65. Interrupt Enable Register Description
     Interrupt Enable                                                             Description
                            Interrupt Enable.
                            Default value: 0
     INT_EN_DRDY
                            0: Data Ready interrupt disabled
                            1: Data Ready interrupt enabled

                            Interrupt Enable.
                            Default value: 0
      INT_EN_FIFO
                            0: FIFO interrupt disabled
                            1: FIFO interrupt enabled

                            Interrupt Enable.
                            Default value: 0
       INT_EN_PW
                            0: Pressure window interrupt disabled
                            1: Pressure window interrupt enabled




MPL3115A2

                                                                                                                                     Sensors
32                                                                                                                Freescale Semiconductor, Inc.
Table 65. Interrupt Enable Register Description
                     Interrupt Enable.
                     Default value: 0
     INT_EN_TW
                     0: Temperature window interrupt disabled
                     1: Temperature window interrupt enabled.

                     Interrupt Enable.
                     Default value: 0
    INT_EN_PTH
                     0: Pressure Threshold interrupt disabled
                     1: Pressure Threshold interrupt enabled.

                     Interrupt Enable.
                     Default value: 0
    INT_EN_TTH
                     0: Temperature Threshold interrupt disabled
                     1: Temperature Threshold interrupt enabled.

                     Interrupt Enable.
                     Default value: 0
    INT_EN_PCHG
                     0: Pressure Change interrupt disabled
                     1: Pressure Change interrupt enabled.

                     Interrupt Enable.
                     Default value: 0
    INT_EN_TCHG
                     0: Temperature Change interrupt disabled
                     1: Temperature Change interrupt enabled.




                                                                   MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                             33
7.17.5 CTRL_REG5 [Interrupt Configuration Register] (0x2A)

Table 66. CTRL_REG5 Register
                7                    6              5               4                3               2            1             0
      R
          INT_CFG_DRDY      INT_CFG_FIFO INT_CFG_PW INT_CFG_TW INT_CFG_PTH INT_CFG_TTH INT_CFG_PCHG INT_CFG_TCHG
      W
  Reset         0                    0              0               0                0               0            0             0


Table 67. Interrupt Configuration Register Descriptions
 Interrupt Configuration                                                           Description
                           INT1/INT2 Configuration. Default value: 0
     INT_CFG_DRDY
                           0: Interrupt is routed to INT2 pin; 1: Interrupt is routed to INT1

                           INT1/INT2 Configuration. Default value: 0
     INT_CFG_FIFO
                           0: Interrupt is routed to INT2 pin; 1: Interrupt is routed to INT1

                           INT1/INT2 Configuration. Default value: 0
     INT_CFG_PW
                           0: Interrupt is routed to INT2 pin; 1: Interrupt is routed to INT1

                           INT1/INT2 Configuration. Default value: 0
      INT_CFG_TW
                           0: Interrupt is routed to INT2 pin; 1: Interrupt is routed to INT1

                           INT1/INT2 Configuration. Default value: 0
     INT_CFG_PTH
                           0: Interrupt is routed to INT2 pin; 1: Interrupt is routed to INT1

                           INT1/INT2 Configuration. Default value: 0
     INT_CFG_TTH
                           0: Interrupt is routed to INT2 pin; 1: Interrupt is routed to INT1

                           INT1/INT2 Configuration. Default value: 0
     INT_CFG_PCHG
                           0: Interrupt is routed to INT2 pin; 1: Interrupt is routed to INT1

                           INT1/INT2 Configuration. Default value: 0
     INT_CFG_TCHG
                           0: Interrupt is routed to INT2 pin; 1: Interrupt is routed to INT1




                                                  Event Flag 0
                           Data Ready
                                                  Event Flag 1
                              FIFO

                     Pressure Threshold
                                                  Event Flag 2                                             INT1

                    Temperature Threshold         Event Flag 3
                                                                               INTERRUPT
                                                  Event Flag 4                CONTROLLER
                      Pressure Window

                    Temperature Window
                                                  Event Flag 5                                             INT2
                                                  Event Flag 6
                      Pressure Change
                                                  Event Flag 7
                    Temperature Change




                                                                               8                 8
                                                                 INT_ENABLE                      INT_CFG


                                            Figure 8. Interrupt Controller Block Diagram




MPL3115A2

                                                                                                                              Sensors
34                                                                                                         Freescale Semiconductor, Inc.
   The system’s interrupt controller uses the corresponding bit field in the CTRL_REG5 register to determine the routing table for
the INT1 and INT2 interrupt pins. If the bit value is logic ‘0’ the functional block’s interrupt is routed to INT2, and if the bit value is
logic ‘1’ then the interrupt is routed to INT1. All interrupts routed to INT1 or INT2 are logically OR’d as illustrated in Figure 9, thus
one or more functional blocks can assert an interrupt pin simultaneously; therefore a host application responding to an interrupt
should read the INT_SOURCE register to determine the appropriate sources of the interrupt.


                                 SRC DR




                          SRC Pressure Change                                                             INT
                                     .
                                                                                 OR
                                     .
                                     .

                            SRC Temp Change



                                                  Figure 9. INT1/INT2 PIN Control Logic

7.18     Offset Correction
  The 2’s complement offset correction registers values are used to trim the temperature, altitude and pressure offsets that might
occur over the life of the product.
  The resolution of the pressure offset register is 4 Pa per bit.

7.18.1 OFF_P (0x2B)
  Pressure user accessible offset trim value expressed as an 8-bit 2's complement number. The user offset registers may be
adjusted to enhance accuracy and optimize the system performance. Range is from -512 to +508 Pa, 4 Pa per LSB.

Table 68. OFF_P Register
                     7                   6               5          4               3              2               1               0
           R
                 OFF_P7            OFF_P6              OFF_P5    OFF_P4         OFF_P3          OFF_P2          OFF_P1         OFF_P0
           W
       Reset         0                   0               0          0               0              0               0               0

Table 69. OFF_P Description
        Name                                                              Description
                         Pressure offset trim value.
  OFF_P7-OFF_P0
                         Default value: 0000_0000.


7.18.2 OFF_T (0x2C)
  Temperature user accessible offset trim value expressed as an 8-bit 2's complement number. The user offset registers may
be adjusted to enhance accuracy and optimize the system performance. Range is from -8 to +7.9375°C, 0.0625°C per LSB.

Table 70. OFF_T Register
                      7                  6                5          4              3               2              1               0
            R
                   OFF_T7           OFF_T6             OFF_T5    OFF_T4          OFF_T3         OFF_T2          OFF_T1         OFF_T0
            W
        Reset         0                  0                0          0              0               0              0               0




                                                                                                                            MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                           35
Table 71. OFF_Y Description
       Name                                                         Description
                    Temperature offset trim value.
  OFF_T7-OFF_T0
                    Default value: 0000_0000.


7.18.3 OFF_H (0x2D)
   Altitude Data User Offset Register (OFF_H) is expressed as a 2’s complement number in meters. The user offset register pro-
vides user adjustment to the vertical height of the Altitude output. The range of values are from -128 to +127 meters.

Table 72. OFF_H Register
                    7              6                 5        4             3             2              1            0
           R
                 OFF_H7        OFF_H6           OFF_H5     OFF_H4        OFF_H3        OFF_H2        OFF_H1        OFF_H0
           W
       Reset        0              0                 0        0             0             0              0            0


Table 73. OFF_Y Description
       Name                                                         Description
                    Height offset trim value.
  OFF_H7-OFF_H0
                    Default value: 0000_0000.




MPL3115A2

                                                                                                                   Sensors
36                                                                                              Freescale Semiconductor, Inc.
8        Soldering/Landing Pad Information
  The LGA package is compliant with the RoHS standard. Note that Pin 1 index area marker does not have any internal electrical
connections. Handling and soldering recommendations for pressure sensors are available in application note AN3150.




                                Figure 10. MPL3115A2 Recommended PCB Landing Pattern.




                                                                                                                 MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                              37
            PACKAGE DIMENSIONS




                CASE 2053-01
                  ISSUE O
               LGA PACKAGE


MPL3115A2

                                                    Sensors
38                               Freescale Semiconductor, Inc.
                                PACKAGE DIMENSIONS




                                    CASE 2053-01
                                      ISSUE O
                                   LGA PACKAGE


                                                     MPL3115A2

Sensors
Freescale Semiconductor, Inc.                               39
9        Tape and Reel Specifications




                                                                               (I)   Measured from centerline of sprocket hole to
     Ao       3.35 ± 0.10                                                            centerline of pocket.
                                                                               (II) Cumulative tolerance of 10 sprocket holes is
     Bo       5.35 ± 0.10
                                                                                     ±0.20.
     Ko       1.20 ± 0.10                                                      (III) Measured from centerline of sprocket hole to
                                                                                     centerline of pocket.
     F        5.50 ± 0.10                                                      (IV) Other material available.
     P1       8.00 ± 0.10                                                      Dimensions are in millimeters.

     W        12.00 ± 0.10




                             Figure 11. LGA (3 by 5) Embossed Carrier Tape Dimensions




                                 Pin 1 Index Area




                                    Figure 12. Device Orientation in Chip Carrier




MPL3115A2

                                                                                                                Sensors
40                                                                                           Freescale Semiconductor, Inc.
Table 74. Revision History

 Revision    Revision
                                                                  Description of changes
 number        date

     0        06/2011    • Initial Release

                         • Added bullet and new row under Ordering Information on pg 1.
                         • Global change to register names 0x16, 0x17, 0x18, 0x19, 0x1A; changed ARM to TGT.
                         • Global change to bit names in registers 0x12 and 0x2A.
                         • Section 2.1, Table 1, changed Min and Max values for Pressure Absolute Accuracy.
                         • Table 3 changed units to ns for SDA Data Hold Time
     1        12/2011    • Added Figure 4. I2C Bus Transmission Signals
                         • Section 4.3: added equation
                         • Section 5: Added new paragraph. Added new equations in 5.6.1 and 5.6.2.
                         • Section 6: Added footnote to Table 11, changed TOW description in Table 12, updated paragraphs of Sections
                           6.1.3 and 6.1.4, Table 57 added sentence in RAW description column, Table 58 added column for Fastest
                           OST Data Output I2C, updated Figure 5 and Figure 6.

                         • Section 2.1, Table 2, changed from: -10°C, to: over -10°C for Pressure Absolute Accuracy parameter, changed
                           parameter name from: Pressure Resolution, to: Pressure/Altitude Resolution, changed Test Conditions name
                           from: Barometric Measurement Mode, to: Barometer Mode and from: Altimeter Measurement Mode, to:
                           Altimeter Mode, added to Table 2, Footnote 1: over 50kPa to 110 kPa, added new Footnote 2: Oversample
                           (OSR) modes internally combine and average samples to reduce noise.
                         • Table 3: removed footnote reference on Table 3 table heading to Footnote 1, added Footnote 2 reference to
                           Highest Speed Mode test condition, added Footnote 3 reference to Highest Resolution Mode test condition,
                         • Table 4: deleted row 8: SDA Valid Time, row 9: SDA Valid Acknowledge Time and Footnote 5. tVD,DAT,
                           changed SDA and SCL Rise Time Max value from: 1000, to: 300.
                         • Sections 6.6.2, 6.6.3, 6.6.4: Added Note.
                         • Added Sections 6.6.7 Data Ready and Section 6.6.8 FIFO Event.
                         • Section 7, Table 9: Changed Register Address column from: 0x00//0x06, to: 0x06/0x00, from: 0x0D//00, to:
                           0x0D, from: 0x0E/01, to: 0x0E/0x01, added OST to Footnote 4.
     2        04/2012    • Section 7.1: Table 11: from (0x0B), to: (0x0D), from: Pressure/Temperature data delta, to: Pressure/
                           Temperature data or FIFO data.
                         • Section 7.3.3: Added (ST bits of CTRL_REG2) to sentence 2., Table 31 added 1 new sentence in
                           F_MODE[1.0] description
                         • Deleted notes from Sections 7.9.1, 7.10.1, 7.11.1 and 7.12.1.
                         • Section 7.17.1, Table 58: OST Description, changed from: ACTV, to: SBYB, Table 59: Delete Column: Fastest
                           OST Data Output Rate I2C, deleted Footnotes 1 and 2, changed values in Minimum Time Between Data
                           Samples column.
                         • Section 7.18.1: Changed from: 512, to: -512 to +508
                         • Section 7.18.2: Changed from: -8, to: +7.9375°C.
                         • Section 7.18.3: Changed from: -128, to: +128 meters.
                         • Section 8: Added: Note that Pin 1 index area marker does not have any internal electrical connections.
                           Handling and soldering recommendations for pressure sensors are available in application note AN3150.
                         • Section 9: Added Section called Tape and Reel specifications.

    2.1       05/2012    • Changed Ordering Information device name from MPL3115A2T1 to MPL3115A2R1

                         • Page 33, Table 59 Change from: 8 ms, To: 18 ms, Page 38, Table 72, Change title from: OFF_T Register, To:
    2.2       07/2012
                           OFF_H Register

                         • Updated data sheet status from Advance Information to Technical Information.
                         • Modified first paragraph: Updated “30 cm output resolution” to “10 cm output resolution”. Deleted paragraph
                           “Pressure output can be resolved with...”
                         • Updated bullet under Features: “Resolution down to 1 ft. / 30 cm” to “Resolution down to 0.1 m”.
    3.0       12/2013
                         • Section 4: Changed title of section, updated paragraph for easier readability, and deleted codes for routines.
                         • Section 7: Changed Comment column for Register Address 0x05 from “Bits 1-3” to “Bits 0-3”. Subsection
                           7.1.3: Updated paragraphs for Altitude, Pressure and Temperature data. Section 7.2 Table 24: Corrected
                           register table.




                                                                                                                            MPL3115A2

Sensors
Freescale Semiconductor, Inc.                                                                                                          41
How to Reach Us:                                             Information in this document is provided solely to enable system and software
Home Page:                                                   implementers to use Freescale products. There are no express or implied copyright
www.freescale.com                                            licenses granted hereunder to design or fabricate any integrated circuits based on the
Web Support:                                                 information in this document.
http://www.freescale.com/support
                                                             Freescale reserves the right to make changes without further notice to any products
                                                             herein. Freescale makes no warranty, representation, or guarantee regarding the
                                                             suitability of its products for any particular purpose, nor does Freescale assume any
                                                             liability arising out of the application or use of any product or circuit, and specifically
                                                             disclaims any and all liability, including without limitation consequential or incidental
                                                             damages. “Typical” parameters that may be provided in Freescale data sheets and/or
                                                             specifications can and do vary in different applications, and actual performance may
                                                             vary over time. All operating parameters, including “typicals,” must be validated for each
                                                             customer application by customer’s technical experts. Freescale does not convey any
                                                             license under its patent rights nor the rights of others. Freescale sells products pursuant
                                                             to standard terms and conditions of sale, which can be found at the following address:
                                                             http://www.reg.net/v2/webservices/Freescale/Docs/TermsandConditions.htm.




                                                             Freescale, the Freescale logo, and the Energy Efficient Solutions logo are trademarks
                                                             of Freescale Semiconductor, Inc., Reg. U.S. Pat. & Tm. Off. Xtrinsic is a trademark of
                                                             Freescale Semiconductor, Inc.

                                                             All other product or service names are the property of their respective owners.

                                                             © 2013 Freescale Semiconductor, Inc. All rights reserved.




                     RoHS-compliant and/or Pb-free versions of Freescale products have the functionality and electrical
                     characteristics of their non-RoHS-compliant and/or non-Pb-free counterparts. For further information,
                     see http:/www.freescale.com or contact your Freescale sales representative.

                     For information on Freescale’s Environmental Products program, go to http://www.freescale.com/epp.




MPL3115A2
Rev 3.0
12/2013
```
