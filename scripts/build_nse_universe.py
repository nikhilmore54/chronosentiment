#!/usr/bin/env python3
"""
ChronoSentiment — NSE Ticker Universe Builder
Fetches official securities listing from NSE India, cleans and formats them for yfinance (.NS),
sorts alphabetically, and chunks them into 500-symbol cohorts.
"""

import urllib.request
import csv
import io
import os
from pathlib import Path

def main():
    print("=" * 60)
    print("  ChronoSentiment — NSE Universe Builder")
    print("=" * 60)
    
    url = "https://nsearchives.nseindia.com/content/equities/EQUITY_L.csv"
    headers = {
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
    }
    
    print(f"📡 Fetching official equities listing from: {url}")
    req = urllib.request.Request(url, headers=headers)
    
    try:
        with urllib.request.urlopen(req, timeout=15) as response:
            csv_data = response.read().decode('utf-8')
        print("✅ Successfully downloaded NSE Equities list.")
    except Exception as e:
        print(f"❌ Error fetching from NSE India: {e}")
        print("Fallback: Generating a robust, diversified offline set of NSE tickers...")
        # Offline high-quality fallback of 550+ common NSE stock symbols
        generate_offline_fallback()
        return

    # Parse CSV data
    symbols = []
    f = io.StringIO(csv_data)
    reader = csv.DictReader(f)
    
    for row in reader:
        sym = row.get("SYMBOL")
        series = row.get("SERIES")
        if sym and series == "EQ": # Standard Equities only
            symbols.append(f"{sym.strip()}.NS")
            
    if not symbols:
        # Fallback if dictionary headers are different
        f.seek(0)
        reader = csv.reader(f)
        header = next(reader)
        # Find symbol index
        sym_idx = 0
        series_idx = None
        for idx, col in enumerate(header):
            if "SYMBOL" in col.upper():
                sym_idx = idx
            if "SERIES" in col.upper():
                series_idx = idx
        
        f.seek(0)
        next(reader) # skip header
        for row in reader:
            if len(row) > sym_idx:
                sym = row[sym_idx].strip()
                series = row[series_idx].strip() if (series_idx is not None and len(row) > series_idx) else "EQ"
                if sym and series == "EQ":
                    symbols.append(f"{sym}.NS")

    # Sort alphabetically
    symbols = sorted(list(set(symbols)))
    print(f"📊 Identified {len(symbols)} active EQ segment securities on NSE.")
    
    save_cohorts(symbols)

def generate_offline_fallback():
    # Diversified high-fidelity fallback list
    symbols = [
        "AARTIDRUGS.NS", "AARTIIND.NS", "AAVAS.NS", "ABB.NS", "ABBOTINDIA.NS", "ABCAPITAL.NS", "ABFRL.NS",
        "ACC.NS", "ACCELYA.NS", "ADANIENT.NS", "ADANIGREEN.NS", "ADANIPORTS.NS", "ADANIPOWER.NS", "ADANITRANS.NS",
        "ADVENZYMES.NS", "AEGISCHEM.NS", "AHLUCONT.NS", "AIAENG.NS", "AJANTPHARM.NS", "ALKYLAMINE.NS", "ALLCARGO.NS",
        "ALOKTEXT.NS", "AMARAJABAT.NS", "AMBER.NS", "AMBUJACEM.NS", "ANANTRAJ.NS", "APARINDS.NS", "APCOTEXIND.NS",
        "APEX.NS", "APLAPOLLO.NS", "APOLLOHOSP.NS", "APOLLOTYRE.NS", "APTUS.NS", "ARCHIDPLY.NS", "ARCHIES.NS",
        "AREMONY.NS", "ARIES.NS", "ARMANFIN.NS", "ARVIND.NS", "ARVINDFASN.NS", "ASAHIINDIA.NS", "ASHIANA.NS",
        "ASHOKA.NS", "ASIANPAINT.NS", "ASTERDM.NS", "ASTRAZEN.NS", "ASTRAL.NS", "ASTEC.NS", "ATUL.NS", "AUBANK.NS",
        "AUROPHARMA.NS", "AVANTIFEED.NS", "AVTNPL.NS", "AXISBANK.NS", "AXISCADES.NS", "BAJAJ-M7.NS", "BAJAJ-AUTO.NS",
        "BAJAJCON.NS", "BAJAJELEC.NS", "BAJAJFINSV.NS", "BAJFINANCE.NS", "BALAJITELE.NS", "BALAMINES.NS", "BALKRISIND.NS",
        "BALMLAWRIE.NS", "BALRAMCHIN.NS", "BANARISUG.NS", "BANDHANBNK.NS", "BANG.NS", "BANKA.NS", "BANKBARODA.NS",
        "BANKINDIA.NS", "BANSWRY.NS", "BASF.NS", "BATAINDIA.NS", "BAYERCROP.NS", "BBL.NS", "BBTC.NS", "BCG.NS",
        "BDL.NS", "BEL.NS", "BEML.NS", "BEPL.NS", "BERGEPAINT.NS", "BFINVEST.NS", "BFUTILITIE.NS", "BGRENERGY.NS",
        "BHAGERIA.NS", "BHANDARI.NS", "BHARATFORG.NS", "BHARATGEAR.NS", "BHARATRAS.NS", "BHARTIARTL.NS", "BHEL.NS",
        "BIGBLOC.NS", "BIOCON.NS", "BIOFIL.NS", "BIRLACABLE.NS", "BIRLACORPN.NS", "BIRLAMONEY.NS", "BIRLASOFT.NS",
        "BLISSGVS.NS", "BLS.NS", "BLUESTARCO.NS", "BODALCHEM.NS", "BOMDYEING.NS", "BOSCHLTD.NS", "BPCL.NS",
        "BRET.NS", "BRIGADE.NS", "BRITANNIA.NS", "BSE.NS", "BSHSL.NS", "BSL.NS", "BUMERANG.NS", "BURNPUR.NS",
        "BUTTERFLY.NS", "BVCL.NS", "BYKE.NS", "CADILAHC.NS", "CALSOFT.NS", "CAMLINFINE.NS", "CAMPUS.NS",
        "CANFINHOME.NS", "CANBK.NS", "CANTABIL.NS", "CAPACITE.NS", "CAPLIPOINT.NS", "CAPTRUST.NS", "CARBORUNIV.NS",
        "CAREERP.NS", "CARERATING.NS", "CASTROLIND.NS", "CCCL.NS", "CCHHL.NS", "CDSL.NS", "CEATLTD.NS", "CELEBRITY.NS",
        "CENTENARY.NS", "CENTEXT.NS", "CENTRALBK.NS", "CENTUM.NS", "CENTURYPLY.NS", "CENTURYTEX.NS", "CERA.NS",
        "CEREBRAINT.NS", "CESC.NS", "CGPOWER.NS", "CGCL.NS", "CHALET.NS", "CHAMBLFERT.NS", "CHEMCON.NS", "CHEMBOND.NS",
        "CHEMFAB.NS", "CHOLAHLDNG.NS", "CHOLAFIN.NS", "CIGNITI.NS", "CINELINE.NS", "CINEVISTA.NS", "CIPLA.NS",
        "CLEAN.NS", "CLSEL.NS", "COALINDIA.NS", "COCHINSHIP.NS", "COFORGE.NS", "COLPAL.NS", "CONCOR.NS", "CONFIPET.NS",
        "CONSOFINVT.NS", "CONTROLPR.NS", "COROMANDEL.NS", "COSMOFIRST.NS", "COUNCODUT.NS", "CRAFTSMAN.NS", "CREATIVE.NS",
        "CREST.NS", "CRISIL.NS", "CROMPTON.NS", "CSBBANK.NS", "CSLFINANCE.NS", "CTE.NS", "CUB.NS", "CUMMINSIND.NS",
        "CUPID.NS", "CYIENT.NS", "DAAWAT.NS", "DABUR.NS", "DALBHARAT.NS", "DALMIASUG.NS", "DAMODARIND.NS", "DANGEE.NS",
        "DATAMATICS.NS", "DATAPATTNS.NS", "DBCORP.NS", "DBL.NS", "DBREALTY.NS", "DCAL.NS", "DCBBANK.NS", "DCM.NS",
        "DCMSHRIRAM.NS", "DCW.NS", "DECCANCE.NS", "DEEPAKFERT.NS", "DEEPAKNTR.NS", "DEEPENR.NS", "DELHIVERY.NS",
        "DELPHIPRO.NS", "DELTACOUP.NS", "DELTACORP.NS", "DEN.NS", "DENORA.NS", "DEVYANI.NS", "DFMFOODS.NS", "DGCONTENT.NS",
        "DHAMPURSUG.NS", "DHANBANK.NS", "DHANI.NS", "DHANUKA.NS", "DHARMAJ.NS", "DHRUV.NS", "DIAMONDYD.NS", "DICIND.NS",
        "DIGISPICE.NS", "DIGJAYCL.NS", "DISHTV.NS", "DIVISLAB.NS", "DIXON.NS", "DLF.NS", "DLINKINDIA.NS", "DMART.NS",
        "DNAMEDIA.NS", "DODLA.NS", "DOLATALGO.NS", "DOLLAR.NS", "DONEAR.NS", "DPABHUSHAN.NS", "DPL.NS", "DPWRES.NS",
        "DREDGECORP.NS", "DRREDDY.NS", "DSPBLACK.NS", "DTIL.NS", "DUCON.NS", "DWARKESH.NS", "DYCL.NS", "DYNAMATIC.NS",
        "EASEMYTRIP.NS", "EASTSILK.NS", "ECLERX.NS", "EDELWEISS.NS", "EDUCOMP.NS", "EICHERMOT.NS", "EIDPARRY.NS",
        "EIHAHOTELS.NS", "EIHOTEL.NS", "EIMCOELECO.NS", "EKC.NS", "ELDEHSG.NS", "ELECON.NS", "ELECTCAST.NS",
        "ELECTHERM.NS", "ELGIEQUIP.NS", "ELGIRUBCO.NS", "EMAMILTD.NS", "EMAMIPAP.NS", "EMAMIREAL.NS", "EMKAY.NS",
        "EMMBI.NS", "ENDURANCE.NS", "ENERGYDEV.NS", "ENGINERSIN.NS", "ENIL.NS", "EPL.NS", "EQUITASBNK.NS", "ERIS.NS",
        "EROSMEDIA.NS", "ESABINDIA.NS", "ESCORTS.NS", "ESSENTIA.NS", "ESTER.NS", "ETHOS.NS", "EVEREADY.NS", "EVERESTIND.NS",
        "EXCEL.NS", "EXCELINDUS.NS", "EXIDEIND.NS", "EXPLEOSOL.NS", "EXPOPLAST.NS", "FACT.NS", "FAIRCHEMOR.NS", "FCL.NS",
        "FCONSUMER.NS", "FDFL.NS", "FEDERALBNK.NS", "FEL.NS", "FEME.NS", "FESC.NS", "FIEMIND.NS", "FILATEX.NS",
        "FINCABLES.NS", "FINEORG.NS", "FINPIPE.NS", "FSL.NS", "GABRIEL.NS", "GAEL.NS", "GAIL.NS", "GALAXYSURF.NS",
        "GALLANTT.NS", "GANDHITUBE.NS", "GANECOS.NS", "GANESHANR.NS", "GANGESSECU.NS", "GARFIBRES.NS", "GATEWAY.NS",
        "GATI.NS", "GAYAPROJ.NS", "GEECEE.NS", "GEEKAY.NS", "GENESYS.NS", "GENUSPAPER.NS", "GENUSPOWER.NS", "GEOJITFSL.NS",
        "GEPIL.NS", "GESHIP.NS", "GET&D.NS", "GFLLIMITED.NS", "GHCL.NS", "GICHSGFIN.NS", "GICRE.NS", "GILLANDERS.NS",
        "GILLETTE.NS", "GINNIFILA.NS", "GIPCL.NS", "GKP.NS", "GKWLIMITED.NS", "GLAND.NS", "GLAXO.NS", "GLENMARK.NS",
        "GLFL.NS", "GLOBAL.NS", "GLOBALVECT.NS", "GLOBUSSPR.NS", "GMBREW.NS", "GMDCLTD.NS", "GMMPFAUDL.NS", "GMRINFRA.NS",
        "GNA.NS", "GNFC.NS", "GOACARBON.NS", "GOCLCORP.NS", "GOCOLORS.NS", "GODFRYPHLP.NS", "GODREJAGRO.NS", "GODREJCP.NS",
        "GODREJIND.NS", "GODREJPROP.NS", "GOENKA.NS", "GOKEX.NS", "GOKUL.NS", "GOKULAGRO.NS", "GOLDENTOBC.NS",
        "GOLDIAM.NS", "GOLDTECH.NS", "GOODLUCK.NS", "GPIL.NS", "GPPL.NS", "GPTINFRA.NS", "GRANULES.NS", "GRAPHITE.NS",
        "GRASIM.NS", "GRAVITA.NS", "GREAVESCOT.NS", "GREENPANEL.NS", "GREENPLY.NS", "GREENPOWER.NS", "GRINDWELL.NS",
        "GRINFRA.NS", "GRSE.NS", "GSFC.NS", "GSPL.NS", "GSS.NS", "GTC.NS", "GTL.NS", "GTLINFRA.NS", "GTPL.NS",
        "GUFICBIO.NS", "GUJALKALI.NS", "GUJAPOLLO.NS", "GUJGASLTD.NS", "GUJRAFFIA.NS", "GULFCOSPCI.NS", "GULFZO.NS",
        "GULFOILLUB.NS", "GULPOLY.NS", "GVKPIL.NS", "HAL.NS", "HAPPSTMNDS.NS", "HARERAYA.NS", "HARIOMPIPE.NS",
        "HARRMALAYA.NS", "HATHWAY.NS", "HATSUN.NS", "HAVELLS.NS", "HAVIT.NS", "HBLPOWER.NS", "HBSL.NS", "HCC.NS",
        "HCG.NS", "HCLTECH.NS", "HDFC.NS", "HDFCAMC.NS", "HDFCBANK.NS", "HDFCLIFE.NS", "HDIL.NS", "HEG.NS",
        "HEIDELBERG.NS", "HELIOSFLEX.NS", "HEMIPROP.NS", "HERANBA.NS", "HEROMOTOCO.NS", "HERCULES.NS", "HESTERBIO.NS",
        "HEXATRADEX.NS", "HFCL.NS", "HGINFRA.NS", "HGS.NS", "HIKAL.NS", "HIL.NS", "HIMATSEKA.NS", "HINDALCO.NS",
        "HINDCOMPOS.NS", "HINDCOPPER.NS", "HINDNATGLS.NS", "HINDOILEXP.NS", "HINDPETRO.NS", "HINDUNILVR.NS",
        "HINDZINC.NS", "HIRECT.NS", "HISARMET.NS", "HLVLTD.NS", "HMT.NS", "HMVL.NS", "HONAUT.NS", "HONDAPOWER.NS",
        "HOVSP.NS", "HPAL.NS", "HPIL.NS", "HPL.NS", "HSCL.NS", "HTMEDIA.NS", "HUBTOWN.NS", "HUDCO.NS", "HUHTAMAKI.NS",
        "HYBRID.NS", "HYG.NS", "IBREALEST.NS", "IBULHSGFIN.NS", "ICDSL.NS", "ICEMAKE.NS", "ICICIBANK.NS", "ICICIGI.NS",
        "ICICILI.NS", "ICICIPRULI.NS", "IEX.NS", "IFBAGRO.NS", "IFBIND.NS", "IFCI.NS", "IFG.NS", "IGARASHI.NS",
        "IGL.NS", "IGPL.NS", "IIFL.NS", "IIFLSEC.NS", "IIFLWAM.NS", "IITL.NS", "IL&FSENGG.NS", "IL&FSTRANS.NS",
        "IMAGICAA.NS", "IMFA.NS", "IMPAL.NS", "INCREDIBLE.NS", "INDBANK.NS", "INDHOTEL.NS", "INDIACEM.NS", "INDIAGLYCO.NS",
        "INDIAMART.NS", "INDIANB.NS", "INDIANCARD.NS", "INDIANHUME.NS", "INDIGO.NS", "INDIGOPNTS.NS", "INDLMETER.NS",
        "INDOCO.NS", "INDORAMA.NS", "INDOSOLAR.NS", "INDOTECH.NS", "INDOTHAI.NS", "INDOSTAR.NS", "INDUSINDBK.NS",
        "INDUSTOWER.NS", "INEOSSTYRO.NS", "INFIBEAM.NS", "INFOBEAN.NS", "INFY.NS", "INGERRAND.NS", "INOXGREEN.NS",
        "INOXWIND.NS", "INSECTICID.NS", "INTELLECT.NS", "INTENTECH.NS", "INTERGLOBE.NS", "IONEXCHANG.NS", "IPCALAB.NS",
        "IRB.NS", "IRCON.NS", "IRCTC.NS", "IRFC.NS", "IRIS.NS", "ISEC.NS", "ISFT.NS", "ISGEC.NS", "ISMTLTD.NS",
        "ITC.NS", "ITDC.NS", "ITDCEM.NS", "ITI.NS", "IVC.NS", "IVP.NS", "IZMO.NS", "J&KBANK.NS", "JAGRAN.NS",
        "JAGSNPHARM.NS", "JAIBALAJI.NS", "JAICORPLTD.NS", "JAIHINDPRO.NS", "JAINSTUDIO.NS", "JAKHARIA.NS",
        "JALAN.NS", "JAMNAAUTO.NS", "JASH.NS", "JAYAGROGN.NS", "JAYBARSP.NS", "JAYNECOIND.NS", "JAYSREETEA.NS",
        "JBCHEPHARM.NS", "JBFIND.NS", "JBMA.NS", "JEL.NS", "JINDALPHOT.NS", "JINDALPOLY.NS", "JINDALSAW.NS",
        "JINDALSTEL.NS", "JINDRILL.NS", "JINDWORLD.NS", "JISLJALEQS.NS", "JITFINFRA.NS", "JKCEMENT.NS", "JKIL.NS",
        "JKLAKSHMI.NS", "JKPAPER.NS", "JKTYRE.NS", "JMA.NS", "JMCPROJECT.NS", "JMFINANCIL.NS", "JOCIL.NS", "JOST.NS",
        "JPASSOCIAT.NS", "JPINFRATEC.NS", "JPPOWER.NS", "JSL.NS", "JSWENERGY.NS", "JSWHL.NS", "JSWSTEEL.NS",
        "JTEKTINDIA.NS", "JUBILANT.NS", "JUBLFOOD.NS", "JUBLINGREA.NS", "JUBLPHARMA.NS", "JUNIPER.NS", "JUSTDIAL.NS",
        "JYOTHYLAB.NS", "JYOTISTRUC.NS", "KABRAEXTRU.NS", "KAJARIACER.NS", "KAKATCEM.NS", "KALPATPOWR.NS",
        "KALYANKJIL.NS", "KALYANIPLT.NS", "KAMATHODA.NS", "KAMDHENU.NS", "KANANIIND.NS", "KANSAINER.NS", "KAPSTON.NS",
        "KARDA.NS", "KARMAENG.NS", "KARURVYSYA.NS", "KAUSHALYA.NS", "KAYA.NS", "KAYNES.NS", "KBCGLOBAL.NS",
        "KCP.NS", "KCPSUGIND.NS", "KDDL.NS", "KEC.NS", "KECL.NS", "KEI.NS", "KELLTONTEC.NS", "KERNEX.NS",
        "KESORAMIND.NS", "KEYCORP.NS", "KGL.NS", "KHAICHEM.NS", "KHAITANLTD.NS", "KHANDSE.NS", "KICL.NS", "KIDU.NS",
        "KILITCH.NS", "KIMS.NS", "KINGFA.NS", "KIOCL.NS", "KIRIINDUS.NS", "KIRLOSBROS.NS", "KIRLOSENG.NS",
        "KIRLOSIND.NS", "KITEX.NS", "KKCL.NS", "KMSMEDI.NS", "KNDENGG.NS", "KNEL.NS", "KNRCON.NS", "KOHINOOR.NS",
        "KOKUYOCMLN.NS", "KOLTEPATIL.NS", "KOPRAN.NS", "KOTAKBANK.NS", "KOTARISUG.NS", "KOTHARIPRO.NS", "KPIGREEN.NS",
        "KPIL.NS", "KPITTECH.NS", "KPRMILL.NS", "KRBL.NS", "KREBSBIO.NS", "KRIDHANINF.NS", "KRISHANA.NS",
        "KRONOX.NS", "KSB.NS", "KSCL.NS", "KSERASERA.NS", "KSK.NS", "KSL.NS", "KTIL.NS", "KUBERAN.NS", "KWALITY.NS",
        "L&TFH.NS", "LALPATHLAB.NS", "LANDMARK.NS", "LAOPALA.NS", "LASA.NS", "LAURUSLABS.NS", "LAXMICHEM.NS",
        "LEEL.NS", "LEMONTREE.NS", "LGBBROSLTD.NS", "LGHL.NS", "LIBAS.NS", "LIBERTSHOE.NS", "LICHSGFIN.NS",
        "LICI.NS", "LIKHITHA.NS", "LINC.NS", "LINCOLN.NS", "LINDEINDIA.NS", "LITL.NS", "LLOYDSENGG.NS", "LLOYDSME.NS",
        "LML.NS", "LODHA.NS", "LOGISEX.NS", "LOKESHMACH.NS", "LOTUSEYE.NS", "LOVABLE.NS", "LPDC.NS", "LT.NS",
        "LTIM.NS", "LTTS.NS", "LUMAXIND.NS", "LUMAXTECH.NS", "LUPIN.NS", "LUXIND.NS", "LYKALABS.NS", "LYPSAGEMS.NS"
    ]
    
    symbols = sorted(list(set(symbols)))
    print(f"📊 offline fallback generated {len(symbols)} high-quality NSE symbols alphabetically.")
    save_cohorts(symbols)

def save_cohorts(symbols):
    cohort_dir = Path("cohorts")
    cohort_dir.mkdir(parents=True, exist_ok=True)
    
    # Chunk into sizes of 500
    chunk_size = 500
    cohort_files = []
    
    for i in range(0, len(symbols), chunk_size):
        chunk = symbols[i : i + chunk_size]
        batch_id = (i // chunk_size) + 1
        outfile = cohort_dir / f"batch_{batch_id:03d}.txt"
        
        with open(outfile, "w") as f:
            for s in chunk:
                f.write(f"{s}\n")
                
        cohort_files.append(outfile)
        print(f"💾 Saved Cohort Batch {batch_id:03d}: {len(chunk)} symbols to {outfile}")
        
    print(f"✅ Universe chunks generated successfully in cohorts/ directory.")

if __name__ == "__main__":
    main()
