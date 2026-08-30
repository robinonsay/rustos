# Extracted text of `SDUC-Host-Implementation-Guideline_Ver1.00.pdf`

> Auto-generated with `pdftotext -layout`. Line-art, images, and image-based pages do not survive extraction; verify anything load-bearing against the source PDF.

```text
      SD Specifications
SDUC Host Implementation
             Guideline
           Version 1.00
       January 25, 2024


This Implementation Guideline is provided as
 an SD Association Simplified Specification:

As it is based on SDUC Card Specification’s
         subsets from the referenced
 SD Specifications provided in Chapter 2



         Technical Committee
         SD Card Association
                                                               ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00


Revision History
           Date          Version                   Changes compared to previous issue
   January 25, 2024       1.00     Initial Version of SDUC Host Implementation Guideline




                                                  i
                                                                         ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00


This Implementation Guideline and Simplified Specification
  This Implementation Guideline includes subsets from several complete versions of the SD Specifications
  provided in the referenced SD specifications in Chapter 2 which are all owned by the SD Card Association.
  Therefore, this Implementation Guideline is provided as an SD Association Simplified Specification which
  means that it is available with the convention and the process of other SD Association Simplified
  Specifications, including its release process to the public, the Simplified Specification’s Disclaimers,
  below, and the Simplified Specification Terms and Conditions indicated before downloading.

Conditions for publication
 Publisher:
  SD Card Association
  5000 Executive Parkway, Suite 302, San Ramon, CA 94583 USA
  Phone: +1 (925) 275-6615 / Fax: +1 (925) 275-6691
  E-mail: help@sdcard.org

 Copyright Holder:
  The SD Card Association

 Notes:
  The copyright of the previous versions of SD Specifications Parts 1 Physical Layer Specification and Part 2
  File System Specification (Versions 1.00 and 1.01) and all corrections or non-material changes thereto are
  owned by SD Group.
  The copyright of material changes to the previous versions of SD Specifications Parts 1 Physical Layer
  Specification and Part 2 File System Specification (Version 1.01) are owned by SD Card Association.

 Disclaimers:
  The Simplified Specification is made available by the SD Card Association (the “SDA”) at
  https://www.sdcard.org/downloads/pls/index.html (the “Site”) and your access to and/or use of the Simplified
  Specification is subject to the SIMPLIFIED SPECIFICATION TERMS AND CONDITIONS (the “Terms”) that
  are displayed by clicking the "Download" button at https://www.sdcard.org/downloads/pls/index.html .

  If you are viewing or have accessed the Simplified Specification via any source, medium, or in any other way
  other than directly from the Site pursuant your acceptance of the Terms, then your access to, viewing of, and/or
  use of the Simplified Specification is in violation of the SDA’s and its licensors’ intellectual property rights.
  Accordingly, unless obtained directly from the Site pursuant to the Terms, immediately cease and desist all
  viewing, using, or accessing the Simplified Specification; destroy any copies of the Simplified Specification in
  your possession, custody or control; and, if you desire access to the Simplified Specification, proceed to the
  Site to obtain access and use of the Simplified Specification in an authorized manner pursuant to the Terms.

  Distribution of the Simplified Specification, other than through the Site, is a violation of the Terms and the
  intellectual property rights of the SDA and its licensors. The only rights granted in the Simplified Specification
  are those expressly granted in the Terms. All rights not expressly granted pursuant to your acceptance of the
  Terms are reserved to the SDA and its licensors. Notice is also hereby provided that notwithstanding any rights
  granted by the Terms, any implementation of the Simplified Specifications or any portions thereof may require
  a separate license from the SDA, SD Group, SD-3C, LLC or other third parties.




                                                        ii
                                                                     ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

Conventions Used in This Document
 Naming Conventions
  ● Some terms are capitalized to distinguish their definition from their common English meaning. Words
    not capitalized have their common English meaning.

 Numbers and Number Bases
  ● Hexadecimal numbers are written with a lower case "h" suffix, e.g., FFFFh and 80h.
  ● Binary numbers are written with a lower case "b" suffix (e.g., 10b).
  ● Binary numbers larger than four digits are written with a space dividing each group of four digits, as in
    1000 0101 0010b.
  ● All other numbers are decimal.

 Key Words
  • May:    Indicates flexibility of choice with no implied recommendation or requirement.
  • Shall:  Indicates a mandatory requirement. Designers shall implement such mandatory
            requirements to ensure interchangeability and to claim conformance with the specification.
  • Should: Indicates a strong recommendation but not a mandatory requirement. Designers should give
            strong consideration to such recommendations, but there is still a choice in implementation.

 Application Notes
  Some sections of this document provide guidance to the host implementers as follows:
   Application Note:
   This is an example of an application note.




                                                    iii
                                                                                                ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00


                                              Table of Contents
1. Purpose: .............................................................................................................................. 1

2. Reference ............................................................................................................................ 2
  2.1 SD Specifications ....................................................................................................................... 2
    2.1.1 Part 1 Physical Layer Simplified Specification Ver.7.10 (March 25, 2020) or later ................. 2
    2.1.2 Part 1 Physical 7.10 Test Guideline Ver1.10 (May 7, 2020) .................................................. 2
    2.1.3 Part 1 Physical 7.10 Test Guideline for Host Ver1.00 [Expected to be released in Q1/24’] .... 2
    2.1.4 Part 2 File System Specification Ver.7.00 (June 12, 2018) ................................................... 2
    2.1.5 Part 2 File System 7.00 Test Guideline Ver.1.00 (May 8, 2019) ............................................ 2
  2.2 Target Host Controllers Driver to be upgraded: ........................................................................... 2
    2.2.1 Part A2 SD Host Controller Specification Ver.3.00 or later .................................................... 2

3. SDUC Card Features........................................................................................................... 3
  3.1 SDUC Card Capacity: Greater than 2TB and up to 128TB......................................................... 3
  3.2 SDUC Card User Area Partition .................................................................................................. 3
  3.3 SDUC Card File System: exFAT file system ................................................................................ 3
  3.4 Differences in Physical Layer Specification ................................................................................. 3
    3.4.1 Address Range Extension .................................................................................................... 3
    3.4.2 Identification of SDUC .......................................................................................................... 3
    3.4.3 Size of Well Written Blocks Extension .................................................................................. 3
    3.4.4 Non-SPI Interface Support ................................................................................................... 3
    3.4.5 Non-Security Feature Support .............................................................................................. 3

4. Host General Information: .................................................................................................. 4

5. SD Host Driver Implementation Guideline: ....................................................................... 5
  5.1 Preface ....................................................................................................................................... 5
  5.2 Mandatory SDUC Support Requirements for all hosts and cards ................................................ 5
    5.2.1 Mutual Identification of SDUC in ACMD41 Initialization: ....................................................... 5
    5.2.2 SDUC Card User Area Capacity (Maximum LBA) Calculation .............................................. 6
    5.2.3 Memory Read/Write Commands Sequence .......................................................................... 7
      5.2.3.1 CMD23 Set Block Count ........................................................................................................... 7
      5.2.3.2 SD Host Controller AutoCMD23 ................................................................................................ 7
      5.2.3.3 Card State Change of CMD12 .................................................................................................. 7
    5.2.4 Erase Commands Sequence ................................................................................................ 8
  5.3 Optional SDUC Support Requirements ....................................................................................... 8
    5.3.1 Command Queue(CQ) Mode Commands Sequence ............................................................ 8
    5.3.2 Number of Written Blocks ..................................................................................................... 9
    5.3.3 Video Speed Class Commands Sequence ........................................................................... 9
    5.3.4 Maximum Busy Time .......................................................................................................... 10
    5.3.5 Non-Support of CPRM........................................................................................................ 10
    5.3.6 No Use of ACMD23: Number of Write Blocks Pre-Erased .................................................. 10

6. SDUC Test Method and Verification Tests ...................................................................... 11
  6.1 Recommended tests and test process for SDUC hosts.............................................................. 11
  6.2 Available Test Tools Information ................................................................................................. 11
    6.2.1 SDUC Protocol Analyzer ..................................................................................................... 11


                                                                         iv
                                                                                       ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

   6.2.2 SDUC Card Emulator .......................................................................................................... 11
 6.3 Test Vendors Information for Verification Test Service ................................................................ 11




                                                                  v
                                                                                        ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00


                                             Table of Figures
Figure 5-1 : Extension of ACMD41 for Over 2TB (Figure 4-75 in Ref 2.1.1) ................................................... 5
Figure 5-2 : Memory Command Sequence (Figure 4-77 in Ref 2.1.1) ............................................................ 7
Figure 5-3 : Erase Command Sequence (Figure 4-78 in Ref 2.1.1) ................................................................ 8
Figure 5-4 : Illustration of Executing Write Task (Figure 4-74 in Ref 2.1.1) ..................................................... 8
Figure 5-5 : ACMD22 Data Block for SDUC Card (Figure 4-79 in Ref 2.1.1) .................................................. 9
Figure 5-6 : CMD20 “Start Recording” Command Sequence (Figure 4-80 in Ref 2.1.1) ................................ 9
Figure 5-7 : FAT Update Command Sequence (Figure 4-81 in Ref 2.1.1) ..................................................... 9
Figure 5-8 : CMD20 “Update DIR” Command Sequence (Figure 4-82 in Ref 2.1.1) .................................... 10
Figure 5-9 : CMD20 “Set Free AU” (Figure 4-83 in Ref 2.1.1) ...................................................................... 10




                                              Table of Tables
Table 5-1 : CSD Register Structure (Table 5-3 in Ref 2.1.1) ........................................................................... 6
Table 5-2 : The CSD Register Fields (CSD Version 3.0) (Table 5.3.4-1 in Ref 2.1.1) .................................... 6
Table 5-3 : Modification of CMD12 (Table 4-77 in Ref 2.1.1) .......................................................................... 8
Table 5-4 : Command Queue Function Commands (class 1) (from Table 4.7.4-1 in Ref 2.1.1) ..................... 9
Table 5-5 : SUS_ADDR Extension in SD Status (Table 4-76 in Ref 2.1.1) .................................................... 10




                                                                   vi
                                                             ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

1. Purpose:
  SDUC Memory card - a new SD memory card capacity range starting from >2TB up to ≤128TB
  was introduced with Part 1 Physical Layer SD Specification Ver 7.00.
  To support this new capacity range, a support of a wider address field in the SD protocol was
  added for SDUC cards.
  SD host drivers need to be updated for communicating with SDUC cards.
  This document provides a summary of what are the changes required on host side to support the
  SDUC card. Its intent is to help SD Host Controllers developers, driver developers and/or OS driver
  developers understand the characteristics of SDUC card in comparison to legacy SD cards in
  relation to their access.
  Note: This document is not replacing the specifications and hosts shall always comply with the SD
  specifications.




                                            1
                                                                 ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

2. Reference
2.1 SD Specifications
 2.1.1 Part 1 Physical Layer Simplified Specification Ver.7.10 (March 25, 2020) or
       later
       List of Sections related to SDUC:
       A) Card State Diagram of CMD12 and CMD22 (Section 4.3: Figure 4-13).
       B) Busy length requirement of SDUC (Section 4.6.2.2: 500ms max busy).
       C) Command Class requirements of SDUC (Section 4.7.3: Table 4-21).
       D) Detailed Command Description of CMD22 (Section 4.7.4 Table 4-24 and Table 4-25).
       E) Card State Transition Table of CMD12 and CMD22 (Section 4.8 Table 4-35).
       F) Over 2TB Extension (Section 4.20).
           SDUC definitions in this section should be prioritized to the definitions for SDXC in prior
sections.
            a) Initialization Command ACMD41 (Figure 4-75 instead of Figure 4-4).
            b) SD Status: SUS_ADDR (Table 4-76 instead of Table 4-44).
            c) Card States of CMD12 (Table 4-77 is a part of Table 4-35).
       G) CSD Ver3.0: 28-bit C_SIZE (Section 5.3.4: Table 5.3.4-1).


 2.1.2 Part 1 Physical 7.10 Test Guideline Ver1.10 (May 7, 2020)
        A) Card test items TG3.7 may be helpful to determine host test items.


 2.1.3 Part 1 Physical 7.10 Test Guideline for Host Ver1.00 [Expected to be
       released in Q1/24’]

 2.1.4 Part 2 File System Specification Ver.7.00 (June 12, 2018)
        A) Volume Structure (Section 5.1: Figure 5.2).
        B) Partition GUID (Section 5.2.1.1).
        C) exFAT File System Layout with GPT (Appendix C.5: Figure A-5).

 2.1.5 Part 2 File System 7.00 Test Guideline Ver.1.00 (May 8, 2019)
        A) Volume Structure Check for exFAT (Section 3.2.7).


2.2 Target Host Controllers Driver to be upgraded:
 2.2.1 Part A2 SD Host Controller Specification Ver.3.00 or later
   Part A2 SD Host Controller Simplified Specification Ver 4.20 is a publicly available document that
   may be referred as well.




                                               2
                                                              ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

3. SDUC Card Features
3.1 SDUC Card Capacity: Greater than 2TB and up to 128TB
      SDUC card LBA is extended to 38-bit block address.


3.2 SDUC Card User Area Partition
      GUID Partition Table (GPT) is adopted to support more than 2TB partition. SDUC card user
      area is configured by single partition by GPT. MBR Partition cannot be used.


3.3 SDUC Card File System: exFAT file system
      128TB maximum capacity is determined by SD exFAT parameters:
      25-bit (32MB cluster size) + 22-bit (ceiling 4M clusters) = 47-bit (128TB)



3.4 Differences in Physical Layer Specification
3.4.1 Address Range Extension
      The existing address range of SDXC or less capacity SD cards access commands (RD/WR,
      Erase etc..) is limited to only 32-bit block addressing. An extension of 6-bit additional address
      enables SDUC card to support up to 128TB. This 6-bit upper address is provided through a
      new command (CMD22) that shall precede each of the card access commands that have 32-
      bit address in the argument.
3.4.2 Identification of SDUC
      Mutual Identification of SDUC is defined by ACMD41: The initialization process of Host and
      SDUC card will be completed only when host sets HO2T bit of ACMD41 to “1” and card
      responds R3 with CO2T = “1”. If host sets HO2T=0 (host that does not support SDUC
      cards), the SDUC initialization process is not completed, and it will be aborted by host
      timeout. Setting of HO2T does not affect cards other than SDUC.


3.4.3 Size of Well Written Blocks Extension
      ACMD22 may be used to check well written blocks of a write command. 32-bit block count is
      extended to 64-bit block count by changing ACMD22 returns 64-bit data.
3.4.4 Non-SPI Interface Support
      SDUC card does not support SPI interface.
3.4.5 Non-Security Feature Support
      SDUC card does not support CPRM Security feature (no CPRM secured partition).




                                             3
                                                               ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

4. Host General Information:
  The SDUC specification was defined in order to eliminate HW changes in the host controller
  implemented based on Part A2 SD Host Controller Standard Specification. It would be also
  applicable to a specific SD Host Controller that has flexibility in commands issuing. All the support
  for SDUC may be done by modification of drivers such as the Operating System native support
  drivers and the SD Host Controllers third party drivers supplied by the SD Host Controller
  developer.
  The SDUC specification is applicable to not only SD bus mode but also UHS-II mode using SD-
  TRAN.
  In case of USB readers for SDUC cards – the reader developer needs to take care (by its internal
  FW) of the SDUC support… as long as the standard mass storage device drivers support >2TB
  storage media (currently supported by Windows), there is no need to do any related updates on
  the host side.
  Card’s Format - Cards are distributed with the format of GPT + exFAT. SDA will publish an updated
  version of its popular SD Formatter, supporting SDUC card (exp on Q1/2024).
  Hosts implementing SD format function require updating to comply with the SDUC file system
  specification (GPT + exFAT).




                                             4
                                                             ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

5. SD Host Driver Implementation Guideline:
5.1 Preface
  This section describes SDUC functions which are different from SDXC for updating SDXC drivers
  to SDUC drivers. Regarding functions that are not described in this section, host should implement
  them same as SDXC.

5.2 Mandatory SDUC Support Requirements for all hosts and cards
5.2.1 Mutual Identification of SDUC in ACMD41 Initialization:
  Support of SDUC between host and card can be mutually identified by using ACMD41. SDUC
  supported hosts is indicated by setting HCS=1 and HO2T=1 in the argument (host does not have
  card capacity information at this moment). SDUC card can start initialization if HCS=1 and
  HO2T=1, and successful completion of initialization is indicated by Busy=1, CCS=1 and CO2T=1
  in R3. Then, host can identify the card as SDUC card. Otherwise, ACMD41 continues Busy=0 and
  the initialization will be aborted by host timeout.
  Notes:
   HCS stands for "Host Capacity Support". HCS=1 indicates that host supports capacity more
    than 2GB.
   HO2T stands for "Host Over 2TB". HO2T=1 indicates that host supports capacity over 2TB.
   CCS stands for "Card Capacity Status". CCS=1 indicates that card capacity is larger than 2GB.
   CO2T stands for "Card Over 2TB". CO2T=1 indicates that card capacity is over 2TB.
  Non-SDUC card ignores Command Bit 35 (argument bit 27) and returns R3 always setting Bit 35
  (status bit 27) to 0, while SDUC card checks HCS=1 and HO2T=1 and then returns R3 with CCS=1
  and CO2T=1.
  ACMD41 COMMAND:




        Figure 5-1 : Extension of ACMD41 for Over 2TB (Figure 4-75 in Ref 2.1.1)



                                            5
                                                             ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

 5.2.2 SDUC Card User Area Capacity (Maximum LBA) Calculation
   C_SIZE in CSD register is used to calculate card user area capacity. C_SIZE field is 22-bit for
   SDHC/SDXC and extended to 28-bit for SDUC.

    SDUC card capacity is calculated by following steps:
    (1) Read CSD Register by CMD9.
    (2) Check CSD Version 3.0 - CSD_STRUCTURE (bit[127:126]) = 2 as shown Table 5-3.
    (3) Read 28-bit C_SIZE (bit [75:48]).
    Calculate User Area Capacity:
    User Area Capacity [KByte] = (C_SIZE +1) * 512KByte
    Maximum LBA [block address] = Memory Capacity [KByte] * 2 - 1




                 Table 5-1 : CSD Register Structure (Table 5-3 in Ref 2.1.1)


         Partial Table of R2 Response (Card Specific Data Register) for CMD9(SEND_CSD) is
         provided in Table 5-2.




         Table 5-2 : The CSD Register Fields (CSD Version 3.0) (Table 5.3.4-1 in Ref 2.1.1)

Notes:
        Fixed 512-byte block length is always used in memory read/write operation.
         READ_BL_LEN = 9 (bit [83:80]) and WRITE_BL_LEN = 9 (bit [25:22]).
        CMD16 does not affect block size of memory read/write operations.
         CMD16 is only used to change data length of CMD42 Lock/Unlock.




                                            6
                                                              ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00



5.2.3 Memory Read/Write Commands Sequence
  SDUC maximum capacity 128TB requires 38-bit block address. Lower 32-bit block address is set
  in the argument of memory access command. To set upper 6-bit address, CMD22 is defined for
  SDUC. CMD22 shall precede EVERY memory access command.
  In case of multi-block command, there are two options to stop transfer: a) setting data length to be
  transferred by CMD23 (SET_BLOCK_COUNT) or b) using CMD12 (STOP_TRANSMISSION).




           Figure 5-2 : Memory Command Sequence (Figure 4-77 in Ref 2.1.1)

 5.2.3.1 CMD23 Set Block Count
  CMD23 is to set the block count to be transferred to a memory read/write command. Card supports
  CMD23 which is indicated by SCR bit33=1. If card supports SDR104, support of CMD23 is defined
  sas mandatory. As CMD12 requires severe timing control especially in SDR104, use of CMD23
  can stop transfer free from the timing dependence. Then use of CMD23 is highly recommended.
  In case of SDUC, CMD23 shall precede CMD22. CMD23 function is not changed. The number of
  blocks to transfer can be specified up to 4,294,967,295 (4G-1) by CMD23.


 5.2.3.2 SD Host Controller AutoCMD23
  AutoCMD23 optional function is defined from SD Host Controller Ver.3.00. If AutoCMD23 is
  enabled, CMD23 is automatically issued by Host Controller before issuing a specified command
  in the Command register.
  In case of SDUC, AutoCMD23 may be enabled on issuing CMD22. However, as this will not be
  regular usage of AutoCMD23, use of AutoCMD23 is not recommended. If ADMA3 is available,
  multiple of SD commands can be issued efficiently by programing in the descriptor.


 5.2.3.3 Card State Change of CMD12
  Non-SDUC card was defined that CMD12 is not accepted in "tran" state and no response is
  returned. When host issues CMD12 to abort card operation (e.g., for error recovery), host needed
  to handle two cases whether card returns response or not. No response would be returned if card
  completed the operation before receiving CMD12.




                                            7
                                                               ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00

  SDUC card is changed to accept CMD12 in "tran" state and return its response keeping in Tran
  state. By this change, host can always expect receipt of CMD12 response on aborting card
  operation by CMD12.




                Table 5-3 : Modification of CMD12 (Table 4-77 in Ref 2.1.1)

5.2.4 Erase Commands Sequence
  CMD22 shall always precede CMD32 and CMD33 that have 32-bit block address in its argument.




            Figure 5-3 : Erase Command Sequence (Figure 4-78 in Ref 2.1.1)

5.3 Optional SDUC Support Requirements
  All the following features and operations (i.e. Command Queue, Speed Class support, etc.) are
  optional for cards and hosts. The following descriptions are relevant only if the host indeed intends
  to use the given features.


5.3.1 Command Queue(CQ) Mode Commands Sequence
  38-bit block Address is designated by Task Submission using CMD44 and CMD45. CMD44 shall
  always precede CMD45. For SDUC, Upper 6-bit start block address field is defined in CMD44
  argument [29:24]. CMD45 sets lower 32-bit start block address in its argument.
  SDUC card Task execution is similar to SDXC card. If a task is ready by checking CMD13, a read
  task is executed by CMD46 and a write task is executed by CMD47.




        Figure 5-4 : Illustration of Executing Write Task (Figure 4-74 in Ref 2.1.1)



                                             8
                                                            ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00




Table 5-4 : Command Queue Function Commands (class 1) (from Table 4.7.4-1 in Ref 2.1.1)

 5.3.2 Number of Written Blocks
   After a write command is executed, ACMD22 may be used to check well written blocks by the write
   command operation. In response to ACMD22, Non-SDUC card returns 32bit+CRC data block,
   while SDUC card returns 64bit+CRC data block. Well written block count is extended to 64-bit.




        Figure 5-5 : ACMD22 Data Block for SDUC Card (Figure 4-79 in Ref 2.1.1)


 5.3.3 Video Speed Class Commands Sequence
   CMD22 shall always precede write commands (CMD24/CMD25) for RU Writes after Start
   Recording (Figure 4-80), FAT Update (Figure 4-81), Update DIR (Figure 4-82) to extend to 38-bit
   block addressing.




   Figure 5-6 : CMD20 “Start Recording” Command Sequence (Figure 4-80 in Ref 2.1.1)




        Figure 5-7 : FAT Update Command Sequence (Figure 4-81 in Ref 2.1.1)




                                           9
                                                            ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00




        Figure 5-8 : CMD20 “Update DIR” Command Sequence (Figure 4-82 in Ref 2.1.1)

  CMD20 "Set Free AU" has ADDR field to specify 512KB unit of address in 22-bit field that is
  equivalent to 32-bit block address. Then CMD22 shall precede CMD20 "Set Free AU" (Figure 4-
  83). The other CMD20 functions do not use ADDR filed.



                Figure 5-9 : CMD20 “Set Free AU” (Figure 4-83 in Ref 2.1.1)


  Suspend and Resume use SUS_ADDR in SD Status. Non-SDUC card uses 22-bit SUS_ADDR in
  SD Status [367:346] that is unit of 512KB, while SDUC card uses 28-bit SUS_ADDR in unit of
  512KB combined by upper 6-bit in SD Status [345:340] and lower 22-bit in SD Status [367:346].




         Table 5-5 : SUS_ADDR Extension in SD Status (Table 4-76 in Ref 2.1.1)

5.3.4 Maximum Busy Time
  SDUC Host requires accepting the write busy indication up to 500ms. Busy length requirement of
  SDUC is same as that of SDXC.
  Host should wait busy long enough time more than 500ms. Long busy timeout would not affect
  card performance because occurrence of busy timeout would be low probability and occurrence
  of an error can be detected much quickly by checking response timeout and CRC Status.


5.3.5 Non-Support of CPRM
  SDUC card does not support CPRM function. SD_SECURITY=0 in SCR [54:52] indicates that
  CPRM is not supported. Host should not issue CPRM commands (except ACMD44) and should
  not refer CPRM parameter fields such as SIZE_OF_PROTECTED_AREA in SD Status [479:448]
  (SDUC card sets this field to 0).


5.3.6 No Use of ACMD23: Number of Write Blocks Pre-Erased
  SDUC card does not support ACMD23, therefore host should not issue ACMD23. If host sends
  ACMD23 to SDUC card, card will either treat it as illegal command error in next R1 or ignore it.




                                          10
                                                               ©Copyright 2024 SD Card Association
SDUC Host Implementation Guideline Version 1.00



6. SDUC Test Method and Verification Tests
6.1 Recommended tests and test process for SDUC hosts
 Host vendors are required to comply with the SDA specifications and responsible to perform a self-
 compliance testing process to assure compliance.
 For the SDUC host compliance tests, related to the card access process, the SDA provides a Test
 Guideline for host (as referenced in section 2.1.3) with recommended tests to be done as part of the
 self-compliance tests.


6.2 Available Test Tools Information
   Important Note: At the publication of this document the following test tools are not available yet.
   Please contact the SDA Office or the SDA’s Approved Labs for update about the availability of the
   test tools. Until those test tools will be available the host driver developers will have to test
   themselves for compliance with the above-mentioned SD specifications using their own test tools
   and methods.
6.2.1 SDUC Protocol Analyzer
  An SD Protocol Analyzer that may recognize the new commands related to SDUC as well as check
  for proper command sequences.
  Further information about this item will be available upon its release.

6.2.2 SDUC Card Emulator
  Further information about this item will be available upon its release.

6.3 Test Vendors Information for Verification Test Service
  Further information about this item will be available upon its release.




                                             11
```
