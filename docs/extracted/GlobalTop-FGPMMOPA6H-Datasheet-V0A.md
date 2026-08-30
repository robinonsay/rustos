# Extracted text of `GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf`

> Auto-generated with `pdftotext -layout`. Line-art, images, and image-based pages do not survive extraction; verify anything load-bearing against the source PDF.

```text
                         GlobalTop Technology Inc.

                         FGPMMOPA6H
                         GPS Standalone Module
                         Data Sheet
                         Revision: V0A




Data Sheet

             The FGPMMOPA6H is a 4th generation stand-alone GPS module with lightning fast TTFF, ultra high
             sensitivity (-165dBm), and low power consumption in a small form factor (16*16*4.7mm)


             This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without prior
             permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.

             Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
                                th
             No.16 Nan-ke 9 Rd, Science-Based Industrial Park, Tainan, 741, Taiwan, R.O.C.
             Tel: +886-6-5051268 / Fax: +886-6-5053381 / Email: sales@gtop-tech.com / Web: www.gtop-tech.com
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        2


Version History

Title:                  GlobalTop FGPMMOPA6H Datasheet
Subtitle:               GPS Module
Doc Type:               Datasheet
  Revision                  Date       Author                                                  Description
     V0A                 2012-1-31     Delano                                                  Preliminary




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A         3


Table of Contents

1. Functional Description ........................................................................................................... 4
     1.1 Overview ....................................................................................................................... 4
     1.2 Highlights and Features ................................................................................................ 5
     1.3 System Block Diagram................................................................................................... 6
     1.4 Multi-tone active interference canceller ...................................................................... 7
     1.5 1PPS .............................................................................................................................. 7
     1.6 AGPS Support for Fast TTFF (EPO™) ............................................................................. 7
     1.7 EASY™ ........................................................................................................................... 7
     1.8 AlwaysLocate™ (Advance Power Periodic Mode) ......................................................... 9
     1.9 Embedded Logger function .......................................................................................... 9
     2.0 Antenna Advisor ........................................................................................................... 9
2. Specifications .......................................................................................................................10
     2.1 Mechanical Dimension ............................................................................................... 10
     2.2 Recommended PCB pad Layout .................................................................................. 11
     2.3 Pin Configuration ........................................................................................................ 12
     2.4 Pin Assignment ........................................................................................................... 13
     2.5 Description of I/O Pin ................................................................................................. 14
     2.6 Specification List ......................................................................................................... 16
     2.7 Absolute Maximum Ratings ........................................................................................ 17
     2.8 Operating Conditions .................................................................................................. 17
     2.9 GPS External Antenna Specification (Recommended) ............................................... 17
3. Protocols ..............................................................................................................................18
     3.1 NMEA Output Sentences ............................................................................................ 18
     3.2 Antenna Status Protocol (Antenna Advisor) ............................................................... 24
     3.3 MTK NMEA Command Protocols ................................................................................ 24
     3.4 Firmware Customization Services............................................................................... 25
4. Reference Design ..................................................................................................................26
     4.1 Reference Design Circuit ............................................................................................. 26
5. Packing and Handling............................................................................................................ 27
     5.1 Moisture Sensitivity .................................................................................................... 27
     5.2 Packing ........................................................................................................................ 28
     5.3 Storage and Floor Life Guideline................................................................................. 30
     5.4 Drying.......................................................................................................................... 30
     5.5 ESD Handling............................................................................................................... 31
6. Reflow Soldering Temperature Profile ...................................................................................32
     6.1 SMT Reflow Soldering Temperature Profile................................................................ 32
     6.2 Manual Soldering ........................................................................................................ 36
7. Contact Information.............................................................................................................. 37




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        4


1. Functional Description

1.1 Overview

The FGPMMOPA6H utilizes the MediaTek new generation GPS Chipset MT3339 that achieves the
industry’s highest level of sensitivity (-165dBm ) and instant Time-to-First Fix (TTFF) with lowest
power consumption for precise GPS signal processing to give the ultra-precise positioning under
low receptive, high velocity conditions.

The module a revision of POT (Patch On Top) GPS Module with an extra embedded function for
external antenna I/O and comes with automatic antenna switching function and short circuit
protection, also features a antenna system called “Antenna Advisor” that helps with the detections
and notifications of different antenna statuses, including active antenna connection, antenna open
circuit and antenna shortage.

Up to 12 multi-tone active interference canceller (ISSCC2011 award), customer can have more
flexibility in system design. Supports up to 210 PRN channels with 66 search channels and 22
simultaneous tracking channels, Module supports various location and navigation applications,
including autonomous GPS,QZSS, SBAS(note) ranging (WAAS, EGNO, GAGAN, MSAS), AGPS.

FGPMMOPA6H is excellent low power consumption characteristic (acquisition 82mW, tracking
66mW), power sensitive devices, especially portable applications, need not worry about operating
time anymore and user can get more fun.

Note: SBAS can only be enabled when update rate is less than or equal to 5Hz.

Application:

            Handheld Device

            Tablet PC/PLB/MID

            M2M application

            Asset management

            Surveillance




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        5


1.2 Highlights and Features

    Built-in 15X15X2.5mm ceramic patch antenna on the top of module

    Ultra-High Sensitivity: -165dBm (w/o patch antenna), up to 45dB C/N of SVs in open sky

         reception.

    High Update Rate: up to 10Hz(note1)

    12 multi-tone active interference canceller note2) [ISSCC 2011 Award -Section 26.5]
                                                                   (




         (http://isscc.org/doc/2011/isscc2011.advanceprogrambooklet_abstracts.pdf )

    High accuracy 1-PPS timing support for Timing Applications (10ns jitter)

    AGPS Support for Fast TTFF (EPO™ Enable 7 days/14 days )

    EASY™ note2): Self-Generated Orbit Prediction for instant positioning fix
                   (




    AlwaysLocate™(note2) Intelligent Algorithm (Advance Power Periodic Mode) for power saving

    Logger function Embedded(note2)

    Automatic antenna switching function

    Antenna Advisor function

    Gtop Firmware Customization Services

    Consumption current(@3.3V):
     •   Acquisition: 25mA Typical
     •   Tracking: 20mA Typical

    E911, RoHS, REACH compliant



         note 1: SBAS can only be enabled when update rate is less than or equal to 5Hz.

         note2: Some features need special firmware or command programmed by customer, please
         refer to G-top “GPS command List”




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        6


1.3 System Block Diagram




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        7


1.4 Multi-tone active interference canceller

Because different application (Wi-Fi, GSM/GPRS, 3G/4G, Bluetooth) are integrated into navigation
system, the harmonic of RF signal will influence the GPS reception, The multi-tone active
interference canceller (abbr: MTAIC) can reject external RF interference which come from other
active components on the main board, to improve the capacity of GPS reception without any needed
HW change in the design. PA6H can cancel up to 12 independent channels interference continuous
wave (CW)



1.5 1PPS

A pulse per second (1 PPS) is an electrical signal that very precisely indicates the start of a second.
Depending on the source, properly operating PPS signals have typical accuracy ranging 10ns.

1 PPS signals are used for precise timekeeping and time measurement. One increasingly common
use is in computer timekeeping, including the NTP protocol. A common use for the PPS signal is to
connect it to a PC using a low-latency, low-jitter wire connection and allow a program to synchronize
to it:

PA6H supply the high accurate 1PPS timing to synchronize to GPS time after 3D-Fix.
A power-on output 1pps is also available for customization firmware settings.




1.6 AGPS Support for Fast TTFF (EPO™)

The AGPS (EPO™) supply the predicated Extended Prediction Orbit data to speed TTFF ,users can
download the EPO data to GPS engine from the FTP server by internet or wireless network ,the GPS
engine will use the EPO data to assist position calculation when the navigation information of
satellites are not enough or weak signal zone . About the detail, please link Gtop website .



1.7 EASY™

The EASY™ is embedded assist system for quick positioning, the GPS engine will calculate and predict
automatically the single emperies ( Max. up to 3 days )when power on ,and save the predict
information into the memory , GPS engine will use these information for positioning if no enough
information from satellites, so the function will be helpful for positioning and TTFF improvement
under indoor or urban condition.




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        8




                                          Figure 1.12-1 EASY System operation

Please refer to the Fig 1.12-1, When GPS device great the satellite information from GPS satellites,
the GPS engine automatically pre-calculate the predict orbit information for 3 days

The GPS device still can quickly do the positioning with EASY™ function under weak GPS signal.




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        9


1.8 AlwaysLocate™ (Advance Power Periodic Mode)

Embedded need to be executed full y all the time , the algorithm can be set by different necessary to
decide the operation level of GPS function , reduce power consumption , it will suffer positing
accuracy to get the target of power saving and extend the usage time of product . (The positioning
accuracy of reporting location < 50m (CEP)




1.9 Embedded Logger function

The Embedded Logger function don’t need host CPU (MCU ) and external flash to handle the
operation , GPS Engine will use internal flash (embedded in GPS chipset ) to log the GPS data (Data
format : UTC, Latitude , longitude, Valid ,Checksum ), the max log days can up to 2 days under
AlwaysLocate™ condition .Note

Note: Data size per log was shrunk from 24 bytes to 15 bytes.




2.0 Antenna Advisor
“Antenna Advisor” is a brand new antenna system available exclusively for PA6H. It is designed to
detect and notify antenna status using software (through proprietary protocol on Chapter 3.2).

Antenna Advisor can detect and notify the following:

    •     Active Antenna Shorted
    •     Using Internal Antenna
    •     Using Active Antenna




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        10


2. Specifications

2.1 Mechanical Dimension
Dimension: (Unit: mm, Tolerance: +/- 0.2mm)




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        11


2.2 Recommended PCB pad Layout
(Unit: mm, Tolerance: 0.1mm)




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        12


2.3 Pin Configuration




                                                        (Top view)




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        13


2.4 Pin Assignment

   Pin      Name                          I/O      Description & Note
    1       VCC                            PI      Main DC Power Input
    2       NRESET                          I      Reset Input, Low Active
    3       GND                            P       Ground
    4       VBACKUP                        PI      Backup Power Input for RTC & Navigation Data Retention
    5       3D-FIX                         O       3D-Fix Indicator
    6       NC                             --      Not Connect
    7       NC                             --      Not Connect
    8       GND                            P       Ground
    9       TX                             O       Serial Data Output for NMEA Output (UART TTL)
   10       RX                              I      Serial Data Input for Firmware Update (UART TTL)
   11       EX_ANT                         I       External active antenna RF input.
                                          PO       DC power from VCC and provide for external active
                                                   antenna.
   12       GND                            P       Ground
   13       1PPS                           O       1PPS Time Mark Output 2.8V CMOS Level
   14       RTCM                            I      Serial Data Input for DGPS RTCM Data Streaming
   15       NC                             --      Not Connect
   16       NC                             --      Not Connect
   17       NC                             --      Not Connect
   18       NC                             --      Not Connect
   19       GND                            P       Ground
   20       NC                             --      Not Connect




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        14


2.5 Description of I/O Pin

VCC (Pin1)

The main DC power supply of the module, the voltage should be kept between from 3.0V to 4.3V.
The Vcc ripple must be controlled under 50mVpp (Typical: 3.3V)



NRESET (Pin2)
With a low level, it causes the module to reset. If not used, keep floating.



GND (Pin3, Pin8, Pin12, Pin19)

Ground


VBACKUP (Pin4)

This connects to the backup power of the GPS module. Power source (such as battery) connected to
this pin will help the GPS chipset in keeping its internal RTC running when the main power source is
turned off. The voltage should be kept between 2.0V~4.3V, Typical 3.0V.

IF VBACKUP power was not reserved, the GPS module will perform a lengthy cold start every time
it is powered-on because previous satellite information is not retained and needs to be re-
transmitted.

If not used, keep open.



3D-FIX (Pin5)

The 3D-FIX is assigned as a fix flag output. The timing behavior of this pin can be configured by
custom firmware for different applications (Example: waking up host MCU). If not used, keep
floating.



         Before 2D Fix
         The pin should continuously output one-second high-level with one-second low-level signal.



                                            1s


                                                            1s

 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        15


         After 2D or 3D Fix
         The pin should continuously output low-level signal.
         Low



NC (Pin6, Pin7, Pin15, Pin16, Pin17, Pin18, Pin20)

Not connect.

TX (Pin9)

This is the UART transmitter of the module. It outputs the GPS information for application.


RX (Pin10)

This is the UART receiver of the module. It is used to receive software commands and firmware
update.


EX_ANT (Pin11)
DC power from VCC and provide for external active antenna (Recommendation: 3.3V).
When a 4mA or higher current is detected, the detect circuit will acknowledge the external antenna
as being present and uses external antenna for reception.
In the event of short circuit occurring at external antenna, the module will limit the drawn current to
a safe level.
The 3.0V for the GPS external antenna is limited to 25mA.
The 3.3V for the GPS external antenna is limited to 28mA.
The 3.6V for the GPS external antenna is limited to 31mA.


1PPS (Pin13)

This pin provides one pulse-per-second output from the module and synchronizes to GPS time.
Keep floating if not used.


RTCM (Pin14)
This pin receive DGPS data of RTCM protocol (TTL level), if not used keep floating’
RTCM is not enabled by default, please consult GlobalTop support to enable this feature.




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        16


2.6 Specification List

                                             Description
 GPS Solution                                MTK MT3339
 Frequency                                   L1, 1575.42MHz
                                             Acquisition: -148dBm, cold start
 Sensitivity1                                Reacquisition: -163dBm, Hot start
                                             Tracking: -165dBm
 Channel                                     66 channels
                                             Hot start: 1 second typical
                                             Warm start: 33 seconds typical
 TTFF
                                             Cold start: 35 seconds typical
                                             (No. of SVs>4, C/N>40dB, PDop<1.5)
                                             Without aid:3.0m (50% CEP)
 Position Accuracy
                                             DGPS(SBAS(WAAS,EGNOS,MSAS)):2.5m (50% CEP)

                                             Without aid : 0.1m/s
 Velocity Accuracy
                                             DGPS(SBAS(WAAS,EGNOS,MSAS,GAGAN)):0.05m/s

 Timing Accuracy
                                             10 ns(Typical )
 (1PPS Output)
 Altitude                                    Maximum 18,000m (60,000 feet)
 Velocity                                    Maximum 515m/s (1000 knots)
 Acceleration                                Maximum 4G
 Update Rate                                 1Hz (default), maximum 10Hz
 Baud Rate                                   9600 bps (default)
 DGPS                                        SBAS(defult) [WAAS, EGNOS, MSAS,GAGAN]
 QZSS                                        Support(Ranging)
 AGPS                                        Support
 Power Supply                                VCC：3.0V to 4.3V；VBACKUP：2.0V to 4.3V
 Current Consumption                         25mA acquisition, 20mA tracking
 Working Temperature                         -40 °C to +85 °C
 Dimension                                   16 x 16x 4.7mm, SMD
 Weight                                      4g




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        17


2.7 Absolute Maximum Ratings

The voltage applied for VCC should not exceed 4.3VDC.

                                            Symbol                Min.               Typ.               Max.                Unit
  Power Supply Voltage                         VCC                 3.0                3.3                 4.3                 V
  Backup battery Voltage                   VBACKUP                 2.0                3.0                 4.3                 V




2.8 Operating Conditions

                                                           Condition              Min.           Typ.           Max.          Unit
  Operation supply Ripple Voltage                                －                  －              －             50           mVpp
  RX0 TTL H Level                                        VCC=3.0~4.3V              2.0             －            VCC               V
  RX0 TTL L Level                                        VCC=3.0~4.3V                0             －             0.8              V
  TX0 TTL H Level                                        VCC=3.0~4.3V              2.4             －             2.8              V
  TX0 TTL L Level                                        VCC=3.0~4.3V                0             －             0.4              V
  Current Consumption @ 3.3V,                              Acquisition              －              25            －             mA
  1Hz Update Rate                                           Tracking                －              20            －             mA
  Backup Power Consumption@ 3V                                25°C                  －              7             －             uA


2.9 GPS External Antenna Specification (Recommended)
It is important that the antenna gets a clear view of the sky and is positioned on a surface level to
the horizon for best results. The following specification has to meet for the use reference design.

  Characteristic                                Specification
  Polarization                                  Right-hand circular polarized
  Frequency Received                            1.57542GHz +/- 1.023MHz
  Power Supply                                  3V to 3.6V
  DC Current                                    4mA ~ 20mA at 3.3V
  Total Gain                                    >+ 15dBi (Two-stage LNA)
  Output VSWR                                   < 2.5
  Impedance                                     50ohm
  Noise Figure                                  < 1.5dB




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        18



3. Protocols

3.1 NMEA Output Sentences

Table-1 lists each of the NMEA output sentences specifically developed and defined by MTK
for use within MTK products

                                        Table-1: NMEA Output Sentence
           Option                                               Description
 GGA                                 Time, position and fix type data.
 GSA                                 GPS receiver operating mode, active satellites used in the
                                     position solution and DOP values.
 GSV                                 The number of GPS satellites in view satellite ID numbers,
                                     elevation, azimuth, and SNR values.
 RMC                                 Time, date, position, course and speed data. Recommended
                                     Minimum Navigation Information.
 VTG                                 Course and speed information relative to the ground.




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        19


GGA—Global Positioning System Fixed Data. Time, Position and fix related
data



Table-2 contains the values for the following example：

$GPGGA,064951.000,2307.1256,N,12016.4438,E,1,8,0.95,39.9,M,17.8,M,,*65

                                                 Table-2: GGA Data Format
          Name                      Example               Units                                 Description
Message ID                        $GPGGA                               GGA protocol header
UTC Time                          064951.000                           hhmmss.sss
Latitude                          2307.1256                            ddmm.mmmm
N/S Indicator                     N                                    N=north or S=south
Longitude                         12016.4438                           dddmm.mmmm
E/W Indicator                     E                                    E=east or W=west
Position           Fix            1                                    See Table-3
Indicator
Satellites Used                   8                                    Range 0 to 14
HDOP                              0.95                                 Horizontal Dilution of Precision
MSL Altitude                      39.9                  meters         Antenna Altitude above/below mean-sea-level
Units                             M                     meters         Units of antenna altitude
Geoidal Separation                17.8                  meters
Units                             M                     meters         Units of geoids separation
Age of Diff. Corr.                                      second         Null fields when DGPS is not used
Checksum                          *65
<CR> <LF>                                                              End of message termination


                                            Table-3: Position Fix Indicator
          Value                                                              Description
              0                   Fix not available
              1                   GPS fix
              2                   Differential GPS fix




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        20


GSA—GNSS DOP and Active Satellites

Table-4 contains the values for the following example：

$GPGSA,A,3,29,21,26,15,18,09,06,10,,,,,2.32,0.95,2.11*00

                                    Table-4: GSA Data Format
       Name                        Example    Units            Description
Message ID                        $GPGSA             GSA protocol header
Mode 1                            A                  See Table-5
Mode 2                            3                  See Table-6
Satellite Used                    29                 SV on Channel 1
Satellite Used                    21                 SV on Channel 2
....                              ….          ….     ....
Satellite Used                                       SV on Channel 12
PDOP                              2.32               Position Dilution of Precision
HDOP                              0.95               Horizontal Dilution of Precision
VDOP                              2.11               Vertical Dilution of Precision
Checksum                          *00
<CR> <LF>                                            End of message termination


                                         Table-5: Mode 1
          Value                                     Description
           M                      Manual—forced to operate in 2D or 3D mode
            A                     2D Automatic—allowed to automatically switch 2D/3D


                                              Table-6: Mode 2
          Value                                                    Description
              1                   Fix not available
              2                   2D (＜4 SVs used)
              3                   3D (≧4 SVs used)




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        21


GSV—GNSS Satellites in View

Table-7 contains the values for the following example：

$GPGSV,3,1,09,29,36,029,42,21,46,314,43,26,44,020,43,15,21,321,39*7D

$GPGSV,3,2,09,18,26,314,40,09,57,170,44,06,20,229,37,10,26,084,37*77
$GPGSV,3,3,09,07,,,26*73

                                     Table-7: GSV Data Format
      Name                        Example    Units                Description
 Message ID                      $GPGSV             GSV protocol header
 Number of                       3                  Range 1 to 3
 Messages                                           (Depending on the number of
                                                    satellites tracked, multiple
                                                    messages of GSV data may be
                                                    required.)
 Message                         1                  Range 1 to 3
 Number1
 Satellites in View              09
 Satellite ID                    29                           Channel 1 (Range 1 to 32)
 Elevation                       36                   degrees Channel 1 (Maximum 90)
 Azimuth                         029                  degrees Channel 1 (True, Range 0 to 359)
 SNR (C/No)                      42                   dBHz    Range 0 to 99,
                                                              (null when not tracking)
 ....                            ….                   ….      ....
 Satellite ID                    15                           Channel 4 (Range 1 to 32)
 Elevation                       21                   degrees Channel 4 (Maximum 90)
 Azimuth                         321                  degrees Channel 4 (True, Range 0 to 359)
 SNR (C/No)                      39                   dBHz    Range 0 to 99,
                                                              (null when not tracking)
 Checksum                        *7D
 <CR> <LF>                                                            End of message termination




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        22


RMC—Recommended Minimum Navigation Information

Table-8 contains the values for the following example：

$GPRMC,064951.000,A,2307.1256,N,12016.4438,E,0.03,165.48,260406,3.05,W,A*2C

                                    Table-8: RMC Data Format
      Name                       Example     Units             Description
Message ID                     $GPRMC               RMC protocol header
UTC Time                       064951.000           hhmmss.sss
Status                         A                    A=data valid or V=data not valid
Latitude                       2307.1256            ddmm.mmmm
N/S Indicator                  N                    N=north or S=south
Longitude                      12016.4438           dddmm.mmmm
E/W Indicator                  E                    E=east or W=west
Speed over
                               0.03                     knots
Ground
Course over                                                             True
                               165.48                   degrees
Ground
Date                           260406                           ddmmyy
                                                                E=east or W=west
Magnetic
                               3.05, W                  degrees (Need GlobalTop Customization
Variation
                                                                Service)
                                                                A= Autonomous mode
Mode                           A                                D= Differential mode
                                                                E= Estimated mode
Checksum                       *2C
<CR> <LF>                                                               End of message termination




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        23


VTG—Course and speed information relative to the ground

Table-9 contains the values for the following example:

$GPVTG,165.48,T,,M,0.03,N,0.06,K,A*37

                                      Table-9: VTG Data Format
       Name                       Example      Units             Description
 Message ID                      $GPVTG               VTG protocol header
 Course                          165.48       degrees Measured heading
 Reference                       T                    True
 Course                                       degrees Measured heading
 Reference                       M                    Magnetic
                                                      (Need GlobalTop Customization
                                                      Service)
 Speed                           0.03         knots   Measured horizontal speed
 Units                           N                    Knots
 Speed                           0.06         km/hr   Measured horizontal speed
 Units                           K                    Kilometers per hour
 Mode                            A                    A= Autonomous mode
                                                      D= Differential mode
                                                      E= Estimated mode
 Checksum                        *06
 <CR> <LF>                                            End of message termination




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        24


3.2 Antenna Status Protocol (Antenna Advisor)
The function is for external active antenna only.

PGTOP—Status of antenna

Table-12 contains the values for the following example:

$PGTOP,11,3 *6F

                                         Table-12: PGACK Data Format
        Name                        Example     Units                  Description
  Message ID                        $PGTOP              Protocol header
  Command ID                             11                            Function Type
  Reference                               3                            Value of antenna status


Example:

$PGTOP,11,value*checksum

Value: 1=>Active Antenna Shorted

           2=>Using Internal Antenna

           3=>Using Active Antenna




3.3 MTK NMEA Command Protocols

Packet Type:

      103 PMTK_CMD_COLD_START

Packet Meaning:

      Cold Start：Don’t use Time, Position, Almanacs and Ephemeris data at re-start.

Example:

      $PMTK103*30<CR><LF>




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        25

3.4 Firmware Customization Services
GlobalTop also offers flexible, value-adding GPS firmware customization services that maximizes the
over system efficiencies and power consumptions. Latest functions like Binary Mode, 1-Sentence
Output, Geo-fencing and Last Position Retention, please see our website at www.gtop-tech.com
under Products / GPS Modules / Software Services for more details.

Note: Not all firmware customization services listed below are supported by module. Please contact
GlobalTop Sales or Technical Support for more details.




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                              Document #
     FGPMMOPA6H Data Sheet                                                                                                Ver. V0A        26


4. Reference Design

This chapter introduces the reference schematic design for the best performance. Additional tips
and cautions on design are well documented on Application Note, which is available upon request.



4.1 Reference Design Circuit




Note:

1.   Ferrite bead L1 is added for power noise reduction.
2.   C1 and C2 bypass should be put near the module.
3.   Damping resistors R2,R3,R4 could be modified based on system application for EMI.
4.   If you need more support and information on antenna implementation, please directly contact
     us at sales@gtop-tech.com for further services.




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        27


5. Packing and Handling

GPS modules, like any other SMD devices, are sensitive to moisture, electrostatic discharge, and
temperature. By following the standards outlined in this document for GlobalTop GPS module
storage and handling, it is possible to reduce the chances of them being damaged during production
set-up. This document will go through the basics on how GlobalTop packages its modules to ensure
they arrive at their destination without any damages and deterioration to performance quality, as
well as some cautionary notes before going through the surface mount process.



        Please read the sections II to V carefully to avoid damages permanent damages due to
        moisture intake



        GPS receiver modules contain highly sensitive electronic circuits and are electronic sensitive
        devices and improper handling without ESD protections may lead to permanent damages to
        the modules. Please read section VI for more details.




5.1 Moisture Sensitivity

GlobalTop GPS modules are moisture sensitive, and must be pre-baked before going through the
solder reflow process. It is important to know that:



GlobalTop GPS modules must complete solder reflow process in 72 hours after pre-baking.



This maximum time is otherwise known as “Floor Life”

If the waiting time has exceeded 72 hours, it is possible for the module to suffer damages during the
solder reflow process such as cracks and delamination of the SMD pads due to excess moisture
pressure.




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        28


5.2 Packing

GlobalTop GPS modules are packed in such a way to ensure the product arrives to SMD factory floor
without any damages.

GPS modules are placed individually on to the packaging tray. The trays will then be stacked and
packaged together.

Included are:

    1. Two packs of desiccant for moisture absorption

    2. One moisture level color coded card for relative humidity percentage.

Each package is then placed inside an antistatic bag (or PE bag) that prevents the modules from
being damaged by electrostatic discharge.




                                             Figure 1: One pack of GPS modules



Each bag is then carefully placed inside two levels of cardboard carton boxes for maximum
protection.




                                                   Figure 2: Box protection


 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        29


The moisture color coded card provides an insight to the relative humidity percentage (RH). When
the GPS modules are taken out, it should be around or lower than 30% RH level.

Outside each electrostatic bag is a caution label for moisture sensitive device.




                       Figure 3: Example of moisture color coded card and caution label




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        30


5.3 Storage and Floor Life Guideline
Since GlobalTop modules must undergo solder-reflow process in 72 hours after it has gone through
pre-baking procedure, therefore if it is not used by then, it is recommended to store the GPS
modules in dry places such as dry cabinet.

The approximate shelf life for GlobalTop GPS modules packages is 6 months from the bag seal date,
when store in a non-condensing storage environment (<30°C/60% RH)


        It is important to note that it is a required process for GlobalTop GPS modules to undergo
        pre-baking procedures, regardless of the storage condition.



5.4 Drying

Because the vapor pressures of moisture inside the GPS modules increase greatly when it is exposed
to high temperature of solder reflow, in order to prevent internal delaminating, cracking of the
devices, or the “popcorn” phenomenon, it is a necessary requirement for GlobalTop GPS module to
undergo pre-baking procedure before any high temperature or solder reflow process.


The recommendation baking time for GlobalTop GPS module is as follows:


     60°C for 8 to 12 hours


Once baked, the module’s floor life will be “reset”, and has additional 72 hours in normal factory
condition to undergo solder reflow process.



        Please limit the number of times the GPS modules undergoes baking processes as repeated
        baking process has an effect of reducing the wetting effectiveness of the SMD pad contacts.
        This applies to all SMT devices.



        Oxidation Risk: Baking SMD packages may cause oxidation and/or intermetallic growth of
        the terminations, which if excessive can result in solderability problems during board
        assembly. The temperature and time for baking SMD packages are therefore limited by
        solderability considerations. The cumulative bake time at a temperature greater than 90°C
        and up to 125°C shall not exceed 96 hours. Bake temperatures higher than 125°C are now
        allowed.



 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        31


5.5 ESD Handling


                Please carefully follow the following precautions to prevent severe damage to
                GPS modules.



GlobalTop GPS modules are sensitive to electrostatic discharges, and thus are Electrostatic Sensitive
Devices (ESD). Careful handling of the GPS modules and in particular to its patch antenna (if included)
and RF_IN pin, must follow the standard ESD safety practices:



     Unless there is a galvanic coupling between the local GND and the PCB GND, then the first
      point of contact when handling the PCB shall always be between the local GND and PCB GND.

     Before working with RF_IN pin, please make sure the GND is connected

     When working with RF_IN pin, do not contact any charges capacitors or materials that can
      easily develop or store charges such as patch antenna, coax cable, soldering iron.

     Please do not touch the mounted patch antenna to prevent electrostatic discharge from the
      RF input

     When soldering RF_IN pin, please make sure to use an ESD safe soldering iron (tip).




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        32


6. Reflow Soldering Temperature Profile
The following reflow temperature profile was evaluated by GlobalTop and has been proven to be
reliable qualitatively. Please contact us beforehand if you plan to solder this component using a
deviated temperature profile as it may cause significant damage to our module and your device.

All the information in this sheet can only be used only for Pb-free manufacturing process.




6.1 SMT Reflow Soldering Temperature Profile
       (Reference Only)
Average ramp-up rate (25 ~ 150°C): 3°C/sec. max.

Average ramp-up rate (270°C to peak): 3°C/sec. max.

Preheat: 175 ± 25°C, 60 ~ 120 seconds

Temperature maintained above 217°C: 60~150 seconds

Peak temperature: 250 +0/-5°C, 20~40 seconds

Ramp-down rate: 6°C/sec. max.

Time 25°C to peak temperature: 8 minutes max.

                                                                                Peak:250+0/-5°C
               °C
                                                      Slop:3°C /sec. max.

                                                        (217°C to peak)                                 Slop:6°C /sec. max.

          217°C


                                    Preheat: 175±5°C                               20 ~ 40 sec.




                                          60 ~120 sec.                              60 ~150 sec.

                       Slop:3°C /sec. max.
            25°C


                                                                                                                  Time (sec)




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                    Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
        GlobalTop Technology                                                                                               Document #
        FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        33

       Details                                                     Suggestions                          Notes

1      Before proceeding with the reflow-                          Pre-bake Time:                       The maximum tolerated
       soldering process, the GPS module must                                                           temperature for the tray is
       be pre-baked.                                               6 Hours @ 60°±5°C or                 100°C.
                                                                   4 Hours @ 70°±5°C                    After the pre-baking
                                                                                                        process, please make sure
                                                                                                        the temperature is
                                                                                                        sufficiently cooled down to
                                                                                                        35°C or below in order to
                                                                                                        prevent any tray
                                                                                                        deformation.

2      Because PCBA (along with the patch                          The parameters of the                Double check to see if the
       antenna) is highly endothermic during                       reflow temperature                   surrounding components
       the reflow-soldering process, extra care                    must be set accordingly              around the GPS module are
       must be paid to the GPS module's solder                     to module’s reflow-                  displaying symptoms of
       joint to see if there are any signs of cold                 soldering temperature                cold weld(ing) or false
       weld(ing) or false welding.                                 profile.                             welding.


3      Special attentions are needed for PCBA                      A loading carrier fixture            If there is any bending or
       board during reflow-soldering to see if                     must be used with PCBA               deformation to the PCBA
       there are any symptoms of bending or                        if the reflow soldering              board, this might causes
       deformation to the PCBA board,                              process is using rail                the PCBA to collide into
       possibility due to the weight of the                        conveyors for the                    one another during the
       module. If so, this will cause concerns at                  production.                          unloading process.
       the latter half of the production process.


4      Before the PCBA is going through the                        The operators must                   If the operator is planning
       reflow-soldering process, the production                    check by eyesight and                to readjust the module
       operators must check by eyesight to see                     readjust the position                position, please do not
       if there are positional offset to the                       before reflow-soldering              touch the patch antenna
       module, because it will be difficult to                     process.                             while the module is hot in
       readjust after the module has gone                                                               order to prevent rotational
       through reflow-soldering process.                                                                offset between the patch
                                                                                                        antenna and module



Note: References to patch antenna is referred to GPS modules with integrated Patch-on-top
antennas (PA/Gms Module Series), and may not be applicable to all GPS modules.




    This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                         prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                      Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
        GlobalTop Technology                                                                                               Document #
        FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        34



       Details                                                     Suggestions                          Notes

5      Before handling the PCBA, they must be                      1. Can use electric fans             It is very easy to cause
       cooled to 35°C or below after they have                     behind the Reflow                    positional offset to the
       gone through the reflow-soldering                           machine to cool them                 module and its patch
       process, in order to prevent positional                     down.                                antenna when handling the
       shift that might occur when the module is                                                        PCBA under high
       still hot.                                                  2. Cooling the PCBA can              temperature.
                                                                   prevent the module
                                                                   from shifting due to
                                                                   fluid effect.

6      1. When separating the PCBA panel into                      1. The blade and the                 1. Test must be performed
       individual pieces using the V-Cut process,                  patch antenna must                   first to determine if V-Cut
       special attentions are needed to ensure                     have a distance gap                  process is going to be used.
       there are sufficient gap between patch                      greater than 0.6mm.                  There must be enough space
       antennas so the patch antennas are not                                                           to ensure the blade and
       in contact with one another.                 2. Do not use patch                                 patch antenna do not touch
                                                    antenna as the leverage                             one another.
       2. If V-Cut process is not available and the point when separating
       pieces must be separated manually,           the panels by hand.                                 2. An uneven amount of
       please make sure the operators are not                                                           manual force applied to the
       using excess force which may cause                                                               separation will likely to
       rotational offset to the patch antennas.                                                         cause positional shift in
                                                                                                        patch antenna and module.

7      When separating panel into individual       Use tray to separate                                 It is possible to chip corner
       pieces during latter half of the production individual pieces.                                   and/or cause a shift in
       process, special attentions are needed to                                                        position if patch antennas
       ensure the patch antennas do not come                                                            come in contact with each
       in contact with one another in order to                                                          other.
       prevent chipped corners or positional
       shifts.




Note: References to patch antenna is referred to GPS modules with integrated Patch-on-top
antennas (PA/Gms Module Series), and may not be applicable to all GPS modules.




    This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                         prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                      Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        35


Other Cautionary Notes on Reflow-Soldering Process:

    1. Module must be pre-baked before going through SMT solder reflow process.

    2. The usage of solder paste should follow “first in first out” principle. Opened solder paste
       needs to be monitored and recorded in a timely fashion (can refer to IPQC for related
       documentation and examples).

    3. Temperature and humidity must be controlled in SMT production line and storage area.
       Temperature of 23°C, 60±5% RH humidity is recommended. (please refer to IPQC for related
       documentation and examples)

    4. When performing solder paste printing, please notice if the amount of solder paste is in
       excess or insufficient, as both conditions may lead to defects such as electrical shortage,
       empty solder and etc.

    5. Make sure the vacuum mouthpiece is able to bear the weight of the GPS module to prevent
       positional shift during the loading process.

    6. Before the PCBA is going through the reflow-soldering process, the operators should check
       by eyesight to see if there are positional offset to the module.

    7. The reflow temperature and its profile data must be measured before the SMT process and
       match the levels and guidelines set by IPQC.

    8. If SMT protection line is running a double-sided process for PCBA, please process GPS
       module during the second pass only to avoid repeated reflow exposures of the GPS module.
       Please contact GlobalTop beforehand if you must process GPS module during the 1st pass of
       double-side process.




    Figure 6.2: Place GPS module right-side up when running reflow-solder process, do not invert.




 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
     GlobalTop Technology                                                                                               Document #
     FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        36


    9. Module must be pre-baked before going through SMT solder reflow process.

    10. The usage of solder paste should follow “first in first out” principle. Opened solder paste
        needs to be monitored and recorded in a timely fashion (can refer to IPQC for related
        documentation and examples).

    11. Temperature and humidity must be controlled in SMT production line and storage area.
        Temperature of 23°C, 60±5% RH humidity is recommended. (please refer to IPQC for related
        documentation and examples)

    12. When performing solder paste printing, please notice if the amount of solder paste is in
        excess or insufficient, as both conditions may lead to defects such as electrical shortage,
        empty solder and etc.

    13. The reflow temperature and its profile data must be measured before the SMT process and
        match the levels and guidelines set by IPQC.



6.2 Manual Soldering

Soldering iron:

Bit Temperature: Under 380°C                    Time: Under 3 sec.

Notes:

    1. Please do not directly touch the soldering pads on the surface of the PCB board, in order to
       prevent further oxidation

    2. The solder paste must be defrosted to room temperature before use so it can return to its
       optimal working temperature. The time required for this procedure is unique and dependent
       on the properties of the solder paste used.

    3. The steel plate must be properly assessed before and after use, so its measurement stays
       strictly within the specification set by SOP.

    4. Please watch out for the spacing between soldering joint, as excess solder may cause
       electrical shortage

    5. Please exercise with caution and do not use extensive amount of flux due to possible siphon
       effects on neighboring components, which may lead to electrical shortage.

    6. Please do not use the heat gun for long periods of time when removing the shielding or
       inner components of the GPS module, as it is very likely to cause a shift to the inner
       components and will leads to electrical shortage.



 This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                      prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                   Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
    GlobalTop Technology                                                                                               Document #
    FGPMMOPA6H Data Sheet                                                                                                 Ver. V0A        37


7. Contact Information


 GlobalTop Technology Inc.

 Address: No.16 Nan-ke 9rd Road Science-based Industrial Park, Tainan 741, Taiwan

 Tel: +886-6-5051268

 Fax: +886-6-5053381

 Website: www.gtop-tech.com

 Email: sales@gtop-tech.com




This document is the exclusive property of GlobalTop Tech Inc. and should not be distributed, reproduced, into any other format without
                     prior permission of GlobalTop Tech Inc. Specifications subject to change without prior notice.
                  Copyright © 2011 GlobalTop Technology Inc. All Rights Reserved.
```
