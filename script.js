function genrow(a) {
  name = a['album'];
  edited = a['edited'];
  deleted = a['deleted'];
  total = a['total']
  return "<tr>\n" +
         "  <td class=\"name-row\">" + name + "</td>\n" +
         "  <td class=\"prog-row\">" + edited + "/" + deleted + "/" + total + "</td>\n" +
         "  <td class=\"bar-row\">\n" +
         "    <progress class=\"edited\" max=\"" + total +
         "\" value=\"" + edited + "\"></progress>\n" +
         "    <progress class=\"deleted\" max=\"" + total +
         "\" value=\"" + deleted + "\"></progress>\n" +
         "  </td>" +
         "</tr>\n";
}

$(document).ready(function() {
  var ip = $("#table_ip");
  var complete = $("#table_complete");
  var totaledited = $("#totaledited");
  var totaldeleted = $("#totaldeleted");
  var label = $("#label");
  var edited = 0;
  var deleted = 0;
  var total = 0;
  for(i=0; i < data.length; i++) {
    for(j=0; j < data[i]['albums'].length; j++) {
      album = data[i]['albums'][j];
      if(album['edited'] == album['total'] - album['deleted']) {
        complete.append(genrow(album));
      } else {
        ip.append(genrow(album));
      }
      edited += album['edited'];
      deleted += album['deleted'];
      total += album['total'];
    }
  }

  totaledited.attr('max', total);
  totaledited.attr('value', edited);
  totaldeleted.attr('max', total);
  totaldeleted.attr('value', deleted);
  label.text(edited + '/' + deleted + '/' + total);
});
