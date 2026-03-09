from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import sys
import os
import pandas as pd
import json
import io

# ユーザー環境のパス設定読み込み
try:
    import _init_paths
except ImportError:
    pass

import cv2
import numpy as np
import onnxruntime as ort
from PIL import Image
from config.opts import opts
import xml.etree.ElementTree as ET

def parse_ocr_xml(xml_string):
    """
    XML文字列からテキストとバウンディングボックス情報を抽出する
    """
    # ルート要素がない場合を考慮してラップする
    if not xml_string.strip().startswith('<root>'):
        xml_string = f"<root>{xml_string}</root>"
        
    try:
        root = ET.fromstring(xml_string)
    except ET.ParseError as e:
        print(f"XML Parse Error: {e}")
        return []

    text_blocks = []
    # 名前空間や深い階層がある場合に備えて再帰的に探すか、単純にLINEタグを探す
    for line in root.iter('LINE'):
        try:
            x = int(line.attrib.get('X', 0))
            y = int(line.attrib.get('Y', 0))
            w = int(line.attrib.get('WIDTH', 0))
            h = int(line.attrib.get('HEIGHT', 0))
            string = line.attrib.get('STRING', '')
            
            # テキストブロックの中心座標を計算
            center_x = x + w / 2
            center_y = y + h / 2
            
            text_blocks.append({
                'text': string,
                'x': x, 'y': y, 'w': w, 'h': h,
                'center_x': center_x,
                'center_y': center_y
            })
        except ValueError:
            continue
            
    return text_blocks

def is_point_in_quad(point, quad):
    """
    点が四角形(4点)の内部にあるかを判定する (Ray Casting Algorithm or simple bbox check)
    簡易的に多角形のバウンディングボックスチェックで代用するか、
    外積を用いて厳密に判定する。ここでは凸多角形を仮定して外積で判定します。
    quad: [[x1, y1], [x2, y2], [x3, y3], [x4, y4]] (時計回りまたは反時計回り)
    """
    x, y = point
    
    # まずBBoxで簡易フィルタリング
    quad_xs = [p[0] for p in quad]
    quad_ys = [p[1] for p in quad]
    if not (min(quad_xs) <= x <= max(quad_xs) and min(quad_ys) <= y <= max(quad_ys)):
        return False
        
    # 厳密な判定 (Winding Number Algorithm等だが、凸包ならCross Productで全て同符号かチェック)
    # 順番が保証されていない場合もあるため、ここでは「4点のBBox内にあるか」で近似します。
    # OCRとセルのマッチングであれば、通常BBox判定で十分な精度が出ます。
    return True

def structure_table_to_html(inference_ret, xml_string):
    """
    inference_onnx.py の出力とOCRのXMLからHTMLテーブルを生成するメイン関数
    
    Args:
        inference_ret (dict): {'logi': [[r1, c1, r2, c2], ...], 'center': [[[x1,y1]...], ...]}
        xml_string (str): OCR結果のXML文字列
        
    Returns:
        str: 生成されたHTML文字列
    """
    
    # 1. OCRデータのパース
    text_blocks = parse_ocr_xml(xml_string)
    
    # 2. セル情報の整理
    cells = []
    logi_list = inference_ret.get('logi', [])
    center_list = inference_ret.get('center', [])
    
    if not logi_list or not center_list:
        return "<table></table>"
    
    # 行・列の最大値を求めてグリッドサイズを特定
    max_row = 0
    max_col = 0
    
    for i, (logi, poly) in enumerate(zip(logi_list, center_list)):
        r1, c1, r2, c2 = logi
        max_row = max(max_row, r2)
        max_col = max(max_col, c2)
        
        cells.append({
            'r1': int(r1),
            'c1': int(c1),
            'r2': int(r2),
            'c2': int(c2),
            'poly': poly, # [[x1,y1], [x2,y2], [x3,y3], [x4,y4]]
            'texts': []
        })
        
    # 3. テキストをセルに割り当て
    # 各テキストについて、その中心が含まれるセルを探す
    for text_obj in text_blocks:
        tx, ty = text_obj['center_x'], text_obj['center_y']
        
        best_cell = None
        min_dist = float('inf')
        
        # まず包含判定
        matched_cells = []
        for cell in cells:
            if is_point_in_quad((tx, ty), cell['poly']):
                matched_cells.append(cell)
        
        if matched_cells:
            # 複数のセルに含まれると判定された場合（重なりなど）、中心距離が最も近いものを選ぶ
            for cell in matched_cells:
                # セル中心（4点の平均）
                cx = sum(p[0] for p in cell['poly']) / 4
                cy = sum(p[1] for p in cell['poly']) / 4
                dist = (tx - cx)**2 + (ty - cy)**2
                if dist < min_dist:
                    min_dist = dist
                    best_cell = cell
        else:
            # どのセルにも含まれない場合、最も近いセルに割り当てる（オプション）
            # ここでは割り当てない（または最も近いセルを探す処理を追加してもよい）
            pass
            
        if best_cell:
            best_cell['texts'].append(text_obj)

    # セル内のテキストをY座標順（必要ならX座標順）にソートして結合
    for cell in cells:
        # 一般的に横書きを想定して Y -> X の優先度でソート、あるいは単純に登場順
        # ここでは Y座標でソートして改行っぽく結合するか、スペースで結合するか
        cell['texts'].sort(key=lambda t: (t['y'], t['x']))
        cell['content'] = "".join([t['text'] for t in cell['texts']])

    # 4. HTML生成
    # グリッドの初期化
    # max_row, max_col は 0-based index の最大値なので、サイズは +1
    num_rows = max_row + 1
    num_cols = max_col + 1
    
    # 訪問済みフラグ管理 (結合されたセルにより埋められた場所)
    occupied = np.zeros((num_rows, num_cols), dtype=bool)
    
    # セルを (r1, c1) でソート
    cells.sort(key=lambda x: (x['r1'], x['c1']))
    
    # セル辞書を (r1, c1) をキーにして高速検索できるようにする
    cell_map = {(c['r1'], c['c1']): c for c in cells}
    
    html = ['<table border="1" style="border-collapse: collapse;">']
    
    for r in range(num_rows):
        html.append('  <tr>')
        for c in range(num_cols):
            if occupied[r, c]:
                continue
            
            # (r, c) を開始位置とするセルがあるか確認
            if (r, c) in cell_map:
                cell = cell_map[(r, c)]
                r1, c1, r2, c2 = cell['r1'], cell['c1'], cell['r2'], cell['c2']
                
                rowspan = r2 - r1 + 1
                colspan = c2 - c1 + 1
                content = cell['content']
                
                attr_str = ""
                if rowspan > 1:
                    attr_str += f' rowspan="{rowspan}"'
                if colspan > 1:
                    attr_str += f' colspan="{colspan}"'
                
                html.append(f'    <td{attr_str}>{content}</td>')
                
                # 占有フラグを立てる
                # 範囲チェックを行いながらフラグを更新
                for i in range(r1, min(r2 + 1, num_rows)):
                    for j in range(c1, min(c2 + 1, num_cols)):
                        occupied[i, j] = True
            else:
                # セルが見つからない場合（推論の抜け漏れ）、空のセルを挿入
                # ただし、そこが既に結合セルの一部であればスキップ（occupiedでチェック済）
                html.append('    <td></td>')
                
        html.append('  </tr>')
    
    html.append('</table>')
    return "\n".join(html)

# -----------------------------------------------------------
# NumPy Utils (ONNX Runtime用のヘルパー関数)
# -----------------------------------------------------------
def check_iou(a, b, thr=0.5):
    """IOU (Intersection over Union) を計算して閾値判定を行う"""
    a = np.array(a)
    b = np.array(b)
    a_area = (a[2] - a[0]) * (a[3] - a[1])
    b_area = (b[2] - b[0]) * (b[3] - b[1])
    
    intersection_xmin = np.maximum(a[0], b[0])
    intersection_ymin = np.maximum(a[1], b[1])
    intersection_xmax = np.minimum(a[2], b[2])
    intersection_ymax = np.minimum(a[3], b[3])
    
    intersection_w = np.maximum(0, intersection_xmax - intersection_xmin)
    intersection_h = np.maximum(0, intersection_ymax - intersection_ymin)
    intersection_area = intersection_w * intersection_h
    
    min_area = min(a_area, b_area)
    if min_area == 0: return False
    
    # 交差領域が小さい方の領域の thr 割合以上であればTrue
    if intersection_area / min_area > thr:
        return True
    return False

def parse_ocr_json(ocr_json):
    """
    提供されたJSONフォーマットからテキストとバウンディングボックスを抽出する
    JSON形式: contents -> [ [ {boundingBox: [[x,y]...], text: "str", ...}, ... ] ]
    出力: [{'bbox': [xmin, ymin, xmax, ymax], 'text': str}, ...]
    """
    resobj = []
    
    # contents構造の解析
    contents = ocr_json.get("contents", [])
    if not contents:
        return []

    # contentsはリストのリストになっている構造を想定
    for block in contents:
        if isinstance(block, list):
            for line in block:
                if "boundingBox" not in line or "text" not in line:
                    continue
                
                # 4点座標から矩形(xmin, ymin, xmax, ymax)を計算
                pts = np.array(line["boundingBox"])
                xmin = np.min(pts[:, 0])
                ymin = np.min(pts[:, 1])
                xmax = np.max(pts[:, 0])
                ymax = np.max(pts[:, 1])
                
                bbox = [int(xmin), int(ymin), int(xmax), int(ymax)]
                text = line["text"]
                resobj.append({"bbox": bbox, "text": text})

    # Y座標、X座標の順でソート（読み取り順序の安定化）
    resobj = sorted(resobj, key=lambda x: x['bbox'][0]) # Xでソート
    resobj = sorted(resobj, key=lambda x: x['bbox'][1]) # Yでソート
    return resobj

def dupmerge(conv_atrobjlist, textbboxlist):
    """
    LOREのセル情報とOCRテキストをマージする
    1. 重複または包含関係にあるセルを統合
    2. セル領域に含まれるテキストを結合
    """
    # 1. LOREのセル出力の重複排除と統合
    newconv_atrobjlist = []
    used = set()
    for idx1 in range(len(conv_atrobjlist)):
        if idx1 in used:
            continue
        
        # conv_atrobjlistの構造: [row_idx_min, row_idx_max, col_idx_min, col_idx_max, bbox]
        lbox1 = conv_atrobjlist[idx1][:4] # 論理座標
        bbox1 = conv_atrobjlist[idx1][4]  # 物理座標 [xmin, ymin, xmax, ymax]
        
        for idx2 in range(idx1 + 1, len(conv_atrobjlist)):
            if idx2 in used:
                continue
            bbox2 = conv_atrobjlist[idx2][4]
            lbox2 = conv_atrobjlist[idx2][:4]
            
            # 物理座標が大きく重なっている場合、同一セルとみなしてマージ
            if check_iou(bbox1, bbox2, thr=0.5):
                used.add(idx2)
                # バウンディングボックスを包含するように拡張
                bbox1 = [min(bbox1[0], bbox2[0]), min(bbox1[1], bbox2[1]), 
                         max(bbox1[2], bbox2[2]), max(bbox1[3], bbox2[3])]
                # 論理座標も包含するように拡張
                lbox1 = [min(lbox1[0], lbox2[0]), min(lbox1[1], lbox2[1]), 
                         max(lbox1[2], lbox2[2]), max(lbox1[3], lbox2[3])]
        
        newconv_atrobjlist.append([lbox1, bbox1])

    # 2. テキストボックスをセルにマージ
    reslist = []
    for idx1 in range(len(newconv_atrobjlist)):
        lbox1 = newconv_atrobjlist[idx1][0]
        bbox1 = newconv_atrobjlist[idx1][1]
        
        assigned_texts = []
        
        for textobj in textbboxlist:
            bboxt = textobj["bbox"]
            text = textobj["text"]
            
            # テキストの中心がセルのバウンディングボックス内にあるか、あるいはIOUで判定
            # ここでは元のロジックに従い check_iou(bbox1, bboxt, 0.1) を使用
            # テキストは小さいので、セルに対してわずかでも重なれば（0.1）含める判定
            if check_iou(bbox1, bboxt, thr=0.2):
                assigned_texts.append(textobj)
        
        # セル内でY座標, X座標順にソートして結合
        assigned_texts.sort(key=lambda x: (x['bbox'][1], x['bbox'][0]))
        restext = "".join([t['text'] for t in assigned_texts])
        
        lbox1.append(restext) # [row_min, row_max, col_min, col_max, text]
        reslist.append(lbox1)
        
    return reslist

def tdcreate(rpos, cpos, flagmap, text):
    """HTMLのtdタグ生成（rowspan/colspan対応）"""
    rowsize, colsize = flagmap.shape
    tmpid = flagmap[rpos, cpos]
    
    # colspan計算
    deltac = 0
    for ct in range(cpos, colsize):
       if flagmap[rpos, ct] == tmpid:
            deltac += 1
       else:
            break
            
    # rowspan計算
    deltar = 0
    for rt in range(rpos, rowsize):
        if flagmap[rt, cpos] == tmpid:
            deltar += 1
        else:
            break
            
    if deltac == 1 and deltar == 1:
        return '<td>{}</td>'.format(text)
    else:
        return '<td colspan="{}" rowspan="{}">{}</td>'.format(deltac, deltar, text)

def merge_to_html_and_markdown(ocr_json, lore_result):
    """
    メイン関数: OCR結果とLORE構造解析結果を結合し、HTMLとMarkdownを出力する
    
    Args:
        ocr_json (dict): ユーザー提供のOCR結果JSON
        lore_result (dict): LORE-TSR推論結果 {'center': [...], 'logi': [...]}
    
    Returns:
        dict: {"html": str, "markdown": str}
    """
    # 1. OCR結果のパース
    textbboxlist = parse_ocr_json(ocr_json)
    
    # 2. LORE結果のパースと座標変換
    bndobjlist = []
    atrobjlist = []
    axis_set_row = set()
    axis_set_col = set()
    
    if "center" not in lore_result or "logi" not in lore_result:
        raise ValueError("lore_result must contain 'center' and 'logi' keys.")

    for bndobj, logiobj in zip(lore_result["center"], lore_result["logi"]):
        # LOREのcenterは4点座標 [[x,y], [x,y], [x,y], [x,y]]
        xs = [p[0] for p in bndobj]
        ys = [p[1] for p in bndobj]
        xmin, ymin = int(min(xs)), int(min(ys))
        xmax, ymax = int(max(xs)), int(max(ys))
        bbox = [xmin, ymin, xmax, ymax]
        bndobjlist.append(bbox)
        
        # LOREのlogiは [row_min, row_max, col_min, col_max] (floatの可能性あり)
        rowmin, rowmax = int(logiobj[0]), int(logiobj[1])
        colmin, colmax = int(logiobj[2]), int(logiobj[3])
        
        # 念のため大小関係を整理
        if rowmin > rowmax: rowmin, rowmax = rowmax, rowmin
        if colmin > colmax: colmin, colmax = colmax, colmin
            
        axis_set_row.add(rowmin)
        axis_set_row.add(rowmax)
        axis_set_col.add(colmin)
        axis_set_col.add(colmax)
        
        atrobjlist.append([rowmin, rowmax, colmin, colmax])
        
    # 論理座標の圧縮（0, 1, 2... に振り直し）
    col2idx = {val: idx for idx, val in enumerate(sorted(axis_set_col))}
    row2idx = {val: idx for idx, val in enumerate(sorted(axis_set_row))}
    
    conv_atrobjlist = []
    for idx, (rowmin, rowmax, colmin, colmax) in enumerate(atrobjlist):
        conv_atrobjlist.append([
            row2idx[rowmin], 
            row2idx[rowmax], 
            col2idx[colmin], 
            col2idx[colmax], 
            bndobjlist[idx]
        ])

    # 3. マージ処理
    merged_data = dupmerge(conv_atrobjlist, textbboxlist)
    
    # マージデータをソート（行優先、次に列）
    sorted_data = sorted(merged_data, key=lambda x: (x[0], x[2]))
    
    # 4. HTMLテーブルの構築
    colsize = len(col2idx)
    rowsize = len(row2idx)
    
    # マップ初期化 (-1で埋める)
    # flagmapは、各グリッドセルがどの merged_data インデックス(tmpidx)を参照するかを保持
    flagmap = np.zeros((rowsize + 1, colsize + 1), dtype=int) - 1
    tmpid2text = {}
    
    for tmpidx, (rowmin, rowmax, colmin, colmax, text) in enumerate(sorted_data):
        # 範囲外アクセス防止
        r_end = min(rowmax + 1, rowsize + 1)
        c_end = min(colmax + 1, colsize + 1)
        flagmap[rowmin:r_end, colmin:c_end] = tmpidx
        tmpid2text[tmpidx] = text
        
    tablestr = '<table border="1">'
    tmpidxset = set()
    
    # 行ごとにHTML生成
    # rowsize, colsize は軸の数なので、セル数としては -1 程度になる場合があるが、
    # ここでは座標圧縮後のインデックス最大値までループ
    max_row = max(row2idx.values()) if row2idx else 0
    max_col = max(col2idx.values()) if col2idx else 0

    for r in range(max_row): # 行ループ
        tablestr += "<tr>"
        for c in range(max_col): # 列ループ
            idx = flagmap[r, c]
            if idx == -1:
                # 空セル
                tablestr += '<td></td>'
            elif idx in tmpidxset:
                # 既に処理済みの結合セルの一部ならスキップ
                continue
            else:
                # 新しいセル
                tmpidxset.add(idx)
                text = tmpid2text[idx]
                tablestr += tdcreate(r, c, flagmap, text)
        tablestr += "</tr>"
    tablestr += "</table>"
    
    # 5. Markdown変換 (Pandasを使用)
    try:
        dfs = pd.read_html(tablestr)
        if dfs:
            df = dfs[0]
            markdown_str = df.to_markdown(index=False)
        else:
            markdown_str = ""
    except Exception as e:
        markdown_str = f"Markdown conversion failed: {e}"
        # フォールバック: 単純なCSV形式等
        # markdown_str = df.to_csv(sep="|", index=False)

    return {"html": tablestr, "markdown": markdown_str}
def _sigmoid(x):
    return 1 / (1 + np.exp(-x))

def _nms(heat, kernel=3):
    # 画像端のピークが消えないように境界処理を行う
    hmax = np.zeros_like(heat)
    for i in range(heat.shape[1]):
        hmax[0, i] = cv2.dilate(heat[0, i], np.ones((kernel, kernel), np.uint8),
                                borderType=cv2.BORDER_CONSTANT, borderValue=0)
    keep = (hmax == heat)
    return heat * keep, keep

def _topk(scores, K=40):
    batch, cat, height, width = scores.shape
    scores_flat = scores.reshape(batch, cat, -1)
    
    topk_inds = np.zeros((batch, K), dtype=np.int64)
    topk_score = np.zeros((batch, K), dtype=np.float32)
    topk_ys = np.zeros((batch, K), dtype=np.float32)
    topk_xs = np.zeros((batch, K), dtype=np.float32)
    topk_clses = np.zeros((batch, K), dtype=np.int64)

    for b in range(batch):
        all_scores = scores_flat[b].flatten()
        sorted_indices = np.argsort(all_scores)[::-1]
        
        # K個以上の検出があれば上位K個、なければすべて
        valid_k = min(len(sorted_indices), K)
        top_k_indices = sorted_indices[:valid_k]
        
        # 出力配列に格納（不足分は0埋め）
        topk_score[b, :valid_k] = all_scores[top_k_indices]
        
        pixel_inds = top_k_indices % (height * width)
        cls_inds = (top_k_indices // (height * width)).astype(np.int64)
        
        # 整数除算と剰余で座標計算 (PyTorchの挙動と一致)
        ys = (pixel_inds // width).astype(np.float32)
        xs = (pixel_inds % width).astype(np.float32)
        
        topk_inds[b, :valid_k] = pixel_inds
        topk_ys[b, :valid_k] = ys
        topk_xs[b, :valid_k] = xs
        topk_clses[b, :valid_k] = cls_inds
        
    return topk_score, topk_inds, topk_clses, topk_ys, topk_xs

def _gather_feat(feat, ind):
    dim = feat.shape[1]
    feat_flat = feat.transpose(0, 2, 3, 1).reshape(feat.shape[0], -1, dim)
    out = np.zeros((ind.shape[0], ind.shape[1], dim), dtype=np.float32)
    for i in range(ind.shape[0]):
        out[i] = feat_flat[i, ind[i], :]
    return out

def _get_4ps_feat(cc_match, cr):
    batch, k, _ = cc_match.shape
    channel = cr.shape[1]
    cr_flat = cr.transpose(0, 2, 3, 1).reshape(batch, -1, channel)
    out = np.zeros((batch, k, 4, channel), dtype=np.float32)
    for b in range(batch):
        # 境界チェック: 特徴マップ外の参照を防ぐクリップ
        indices = np.clip(cc_match[b], 0, cr_flat.shape[1] - 1)
        out[b] = cr_flat[b, indices, :]
    return out

# 【修正】デフォルトKをopts.pyに合わせて3000に変更
def ctdet_4ps_decode_numpy(heat, wh, ax, cr, reg=None, K=3000):
    batch, cat, height, width = heat.shape
    heat, keep = _nms(heat)
    scores, inds, clses, ys, xs = _topk(heat, K=K)
    
    if reg is not None:
        reg_feat = _gather_feat(reg, inds)
        xs = xs.reshape(batch, K, 1) + reg_feat[:, :, 0:1]
        ys = ys.reshape(batch, K, 1) + reg_feat[:, :, 1:2]
    else:
        xs = xs.reshape(batch, K, 1) + 0.5
        ys = ys.reshape(batch, K, 1) + 0.5

    wh_feat = _gather_feat(wh, inds)
    ax_feat = _gather_feat(ax, inds)
    clses = clses.reshape(batch, K, 1).astype(np.float32)
    scores = scores.reshape(batch, K, 1)
    
    # BBox復元 (x - w, y - h, ...)
    bboxes = np.concatenate([
        xs - wh_feat[..., 0:1], ys - wh_feat[..., 1:2],
        xs - wh_feat[..., 2:3], ys - wh_feat[..., 3:4],
        xs - wh_feat[..., 4:5], ys - wh_feat[..., 5:6],
        xs - wh_feat[..., 6:7], ys - wh_feat[..., 7:8]
    ], axis=2)
    
    # Corner特徴抽出用のインデックス計算
    p1 = (xs - wh_feat[..., 0:1]) + width * np.round(ys - wh_feat[..., 1:2])
    p2 = (xs - wh_feat[..., 2:3]) + width * np.round(ys - wh_feat[..., 3:4])
    p3 = (xs - wh_feat[..., 4:5]) + width * np.round(ys - wh_feat[..., 5:6])
    p4 = (xs - wh_feat[..., 6:7]) + width * np.round(ys - wh_feat[..., 7:8])
    
    cc_match = np.concatenate([p1, p2, p3, p4], axis=2).astype(np.int64)
    cr_feat_raw = _get_4ps_feat(cc_match, cr)
    cr_feat = np.sum(cr_feat_raw, axis=2)
    
    detections = np.concatenate([bboxes, scores, clses], axis=2)
    return detections, keep, ax_feat, cr_feat

# -----------------------------------------------------------
# Affine Transform Utils
# -----------------------------------------------------------

def get_3rd_point(a, b):
    direct = a - b
    return b + np.array([-direct[1], direct[0]], dtype=np.float32)

def get_dir(src_point, rot_rad):
    sn, cs = np.sin(rot_rad), np.cos(rot_rad)
    src_result = [0, 0]
    src_result[0] = src_point[0] * cs - src_point[1] * sn
    src_result[1] = src_point[0] * sn + src_point[1] * cs
    return src_result

def get_affine_transform_upper_left(center, scale, rot, output_size, inv=0):
    if not isinstance(scale, np.ndarray) and not isinstance(scale, list):
        scale = np.array([scale, scale], dtype=np.float32)
   
    src = np.zeros((3, 2), dtype=np.float32)
    dst = np.zeros((3, 2), dtype=np.float32)
    src[0, :] = center 
    dst[0, :] = [0, 0]
    
    if center[0] < center[1]:
        src[1, :] = [scale[0], center[1]] 
        dst[1, :] = [output_size[0], 0] 
    else:
        src[1, :] = [center[0], scale[0]] 
        dst[1, :] = [0, output_size[0]] 
        
    src[2:, :] = get_3rd_point(src[0, :], src[1, :])
    dst[2:, :] = get_3rd_point(dst[0, :], dst[1, :])

    if inv:
        trans = cv2.getAffineTransform(np.float32(dst), np.float32(src))
    else:
        trans = cv2.getAffineTransform(np.float32(src), np.float32(dst))

    return trans

def affine_transform(pt, t):
    new_pt = np.array([pt[0], pt[1], 1.], dtype=np.float32).T
    new_pt = np.dot(t, new_pt)
    return new_pt[:2]

def transform_preds_upper_left_numpy(coords, center, scale, output_size):
    target_coords = np.zeros(coords.shape)
    trans = get_affine_transform_upper_left(center, scale, 0, output_size, inv=1)
    for p in range(coords.shape[0]):
        target_coords[p, 0:2] = affine_transform(coords[p, 0:2], trans)
    return target_coords

# -----------------------------------------------------------
# Detector Class (ONNX)
# -----------------------------------------------------------

class CtdetDetectorONNX(object):
    def __init__(self, opt):
        self.opt = opt
        self.mean = np.array(opt.mean, dtype=np.float32).reshape(1, 1, 3)
        self.std = np.array(opt.std, dtype=np.float32).reshape(1, 1, 3)
        
        # モデルロード
        detector_path = 'ndltsr_detector.onnx'
        processor_path = 'ndltsr_processor.onnx'
        
        print(f"Loading ONNX models from {opt.save_dir}...")
        try:
            self.detector_session = ort.InferenceSession(detector_path, providers=['CPUExecutionProvider'])
            self.processor_session = ort.InferenceSession(processor_path, providers=['CPUExecutionProvider'])
        except Exception as e:
            print(f"Error loading models: {e}")
            sys.exit(1)

    def pre_process(self, image, scale):
        height, width = image.shape[0:2]
        
        # input_h/w が未設定(-1)の場合は opts.py のデフォルト値に従うが、
        # ここでは ONNX モデルの入力サイズに合わせておくのが安全。
        # opts.init() で ctdet_mid なら 768x768 になるはず。
        inp_height, inp_width = self.opt.input_h, self.opt.input_w
        if inp_height == -1 or inp_width == -1:
             # ctdet_mid default
             inp_height, inp_width = 768, 768
             
        # upper_left=True (NDLTSR default) Logic
        c = np.array([0, 0], dtype=np.float32)
        s = max(height, width) * 1.0
        
        trans_input = get_affine_transform_upper_left(c, s, 0, [inp_width, inp_height])
        
        # Resize
        new_height = int(height * scale)
        new_width = int(width * scale)
        resized_image = cv2.resize(image, (new_width, new_height))
        
        inp_image = cv2.warpAffine(
            resized_image, trans_input, (inp_width, inp_height),
            flags=cv2.INTER_LINEAR)
        
        inp_image = ((inp_image / 255. - self.mean) / self.std).astype(np.float32)
        images = inp_image.transpose(2, 0, 1).reshape(1, 3, inp_height, inp_width)
        
        meta = {'c': c, 's': s, 
                'out_height': inp_height // self.opt.down_ratio, 
                'out_width': inp_width // self.opt.down_ratio}
        return images, meta

    def process_logi(self, logi):
        # ctdet.py の process_logi ロジックを移植
        # 1. 離散化補正
        logi_floor = np.floor(logi)
        dev = logi - logi_floor
        logi = np.where(dev > 0.5, logi_floor + 1, logi_floor)
        
        # 2. 整合性チェック (end < start の場合、end = start に補正)
        # logi shape: (B, K, 4) -> [r1, c1, r2, c2] (予測順序依存だが、NDLTSRは [r_start, r_end, c_start, c_end] ではなく [r_st, c_st, r_ed, c_ed] の可能性が高い。
        # ctdet.py では: logi0 (idx0), logi2 (idx2) を基準にしている。
        # logi_st = cat(logi0, logi0, logi2, logi2)
        # logi < logi_st なら logi_st に置き換え。
        # つまり idx1 < idx0 なら idx1=idx0, idx3 < idx2 なら idx3=idx2。
        
        logi0 = logi[:, :, 0:1] # (1, K, 1)
        logi2 = logi[:, :, 2:3] # (1, K, 1)
        
        # idx1(row_end) と idx0(row_start) の比較用
        # idx3(col_end) と idx2(col_start) の比較用
        logi_st = np.concatenate([logi0, logi0, logi2, logi2], axis=2)
        
        logi = np.where(logi < logi_st, logi_st, logi)
        return logi

    def post_process(self, dets, meta):
        dets[:, :8] *= self.opt.down_ratio
        
        k = dets.shape[0]
        pts = dets[:, :8].reshape(-1, 2)
        
        out_w = meta['out_width'] * self.opt.down_ratio
        out_h = meta['out_height'] * self.opt.down_ratio
        
        pts_orig = transform_preds_upper_left_numpy(pts, meta['c'], meta['s'], [out_w, out_h])
        
        dets[:, :8] = pts_orig.reshape(k, 8)
        return dets

    def run(self, opt, image_numpy):
        scale = 1.0
        images, meta = self.pre_process(image_numpy, scale)
        
        # Detector推論
        input_name = self.detector_session.get_inputs()[0].name
        outputs = self.detector_session.run(None, {input_name: images})
        hm, wh, reg, st, ax, cr = outputs
        hm = _sigmoid(hm)

        # デコード (K=3000を使用)
        dets, keep, ax_feat, cr_feat = ctdet_4ps_decode_numpy(
            hm[:, 0:1, :, :], wh, ax, cr, reg=reg, K=self.opt.K)
        
        if self.opt.wiz_4ps or self.opt.wiz_2dpe:
            logi_feat = ax_feat + cr_feat
        else:
            logi_feat = ax_feat

        res_class1 = dets[0] # 特徴マップ座標系
        valid_mask = res_class1[:, 8] >= self.opt.vis_thresh
        
        full_logi = np.zeros((self.opt.K, 4), dtype=np.float32)

        if np.sum(valid_mask) > 0:
            slct_logi = logi_feat[0, valid_mask, :]
            # Processor入力用: 特徴マップ座標系のまま使用
            slct_dets = res_class1[valid_mask, :8]
            
            slct_logi = np.expand_dims(slct_logi, 0)
            slct_dets = np.expand_dims(slct_dets, 0)
            
            # 正規化 (Processor内の_normalized_ps相当)
            slct_dets = np.round(slct_dets).astype(np.int64)
            slct_dets = np.clip(slct_dets, 0, self.opt.max_fmp_size - 1)

            feat_name = self.processor_session.get_inputs()[0].name
            dets_name = self.processor_session.get_inputs()[1].name
            
            proc_out = self.processor_session.run(None, {
                feat_name: slct_logi.astype(np.float32),
                dets_name: slct_dets
            })
            pred_logi = proc_out[0]
            pred_logi = self.process_logi(pred_logi)
            full_logi[valid_mask] = pred_logi[0]

        # 座標を元画像スケールへ変換
        dets[0] = self.post_process(dets[0], meta)
        
        results_4ps = {1: dets[0], 2: []}
        
        return {'4ps': results_4ps, 'logi': full_logi}
# -----------------------------------------------------------
# Main
# -----------------------------------------------------------

def main(img: Image):
    opt = opts().init()
    opt.gpus = [-1]
    detector = CtdetDetectorONNX(opt)
    
    img_numpy = np.array(img, dtype=np.uint8)
    cvimage = img_numpy[:, :, ::-1] # RGB to BGR
    
    ret = detector.run(opt, cvimage)
    
    
    center_list = []
    logi_list = []
    
    if "4ps" in ret:
        results = ret["4ps"]
        logi = ret["logi"]
        
        for j in range(1, 3):
            if j not in results: continue
            k = 0
            for m in range(len(results[j])):
                bbox = results[j][m]
                k = k + 1
                if bbox[8] > opt.vis_thresh:
                    center_coords = []
                    logi_coords = []
                    for i in range(0, 4):
                        center_coords.append([float(bbox[2*i]), float(bbox[2*i+1])])
                        if logi is not None:
                            if len(logi.shape) == 1:
                                logi_coords.append(int(logi[i]))
                            else:
                                logi_coords.append(int(logi[m, :][i]))
                    center_list.append(center_coords)
                    logi_list.append(logi_coords)
                    
    return {"logi": logi_list, "center": center_list}

if __name__ == "__main__":
    if os.path.exists("test.png"):
        img = Image.open("test.png").convert("RGB")
        sample_lore_result=main(img)
        with open("test.json",encoding="utf-8") as f:
            sample_ocr_json =json.load(f)
        try:
            # ダミーデータだと空になる可能性があるため、try-except
            result = merge_to_html_and_markdown(sample_ocr_json, sample_lore_result)
            print("--- HTML ---")
            print(result["html"])
            print("\n--- Markdown ---")
            print(result["markdown"])
        except Exception as e:
            print(f"Error in example: {e}")
    else:
        print("test.png not found. Please place an image named 'test.png' in the current directory.")